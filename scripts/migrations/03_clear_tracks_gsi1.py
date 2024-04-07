#!/usr/bin/env python3

import os
import sys
from itertools import batched

import asyncio
from aiodynamo.client import Client
from aiodynamo.credentials import EnvironmentCredentials
from aiodynamo.expressions import F, HashKey, RangeKey
from aiodynamo.operations import Update
from aiodynamo.http.httpx import HTTPX
from httpx import AsyncClient
from yarl import URL


async def init_client(h) -> Client:
    endpoint = None
    if os.environ.get("USE_LOCALSTACK") is not None:
        os.environ["AWS_ACCESS_KEY_ID"] = "local"
        os.environ["AWS_SECRET_ACCESS_KEY"] = "local"
        endpoint = URL("http://localhost:4566")

    return Client(
        HTTPX(h),
        credentials=EnvironmentCredentials(),
        region="ap-southeast-1",
        endpoint=endpoint,
    )


async def main():
    if len(sys.argv) != 3:
        print(f"usage: {sys.argv[0]} [table_name] [station_id]")
        exit(1)

    table_name = sys.argv[1]
    station_id = sys.argv[2]

    async with AsyncClient() as h:
        client = await init_client(h)
        table = client.table(table_name)

        print(f"processing station {station_id}")

        items = []
        async for item in table.query(
            key_condition=(
                HashKey("pk", f"STATION#{station_id}#TRACKS")
                & RangeKey("sk").begins_with("TRACK#")
            ),
            filter_expression=F("gsi1pk").exists() | F("gsi1sk").exists(),
            projection=F("pk") & F("sk") & F("updated_ts"),
        ):
            print(f'item sk={item["sk"]} has gsi1 attributes')
            items.append(item)

        print("building transaction items")

        transaction_items = [
            Update(
                table=table_name,
                key={"pk": item["pk"], "sk": item["sk"]},
                expression=F("gsi1pk").remove() & F("gsi1sk").remove(),
                condition=F("updated_ts").equals(item["updated_ts"]),
            )
            for item in items
        ]

        print(f"got {len(items)} to update")

        for chunk in batched(transaction_items, 100):
            print(f"committing a chunk of {len(chunk)}...")
            await client.transact_write_items(chunk)

    print("done")


if __name__ == "__main__":
    asyncio.run(main())

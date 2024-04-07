#!/usr/bin/env python3

import os
import sys
from itertools import batched

import asyncio
from aiodynamo.client import Client
from aiodynamo.credentials import EnvironmentCredentials
from aiodynamo.expressions import F, HashKey, RangeKey
from aiodynamo.models import BatchWriteRequest
from aiodynamo.operations import Update
from aiodynamo.http.httpx import HTTPX
from aiodynamo.types import Item
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

        items = [
            item
            async for item in table.query(
                key_condition=(
                    HashKey("pk", f"STATION#{station_id}#TRACKS")
                    & RangeKey("sk").begins_with("TRACK#")
                ),
                projection=F("id") & F("artist") & F("title"),
            )
        ]

        print(f"got {len(items)}")
        print("building puts")

        puts = [
            {
                "pk": f"STATION#{station_id}#ARTIST#{item['artist']}",
                "sk": f"TITLE#{item['title']}",
                "track_id": item["id"],
            }
            for item in items
        ]

        for chunk in batched(puts, 25):
            put_chunk = list(chunk)
            print(f"batch write a chunk of {len(chunk)}...")
            await client.batch_write(
                {table_name: BatchWriteRequest(items_to_put=put_chunk)}
            )
            # print(put_chunk)

    print("done")


if __name__ == "__main__":
    asyncio.run(main())

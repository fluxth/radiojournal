#!/usr/bin/env python3

import os
import sys
from itertools import batched
from collections import defaultdict
from datetime import UTC, datetime, timedelta

import asyncio
from aiodynamo.client import Client
from aiodynamo.credentials import EnvironmentCredentials
from aiodynamo.expressions import F, HashKey, RangeKey
from aiodynamo.operations import Update
from aiodynamo.http.httpx import HTTPX
from httpx import AsyncClient
from yarl import URL
import ulid


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
        station = await table.get_item(
            {"pk": "STATIONS", "sk": f"STATION#{station_id}"}
        )

        current_dt = datetime.now(tz=UTC).replace(
            hour=0, minute=0, second=0, microsecond=0
        )

        end_dt = (
            ulid.parse(station["first_play_id"])
            .timestamp()
            .datetime.replace(hour=0, minute=0, second=0, microsecond=0)
        )

        print(f'First play partition is {end_dt.strftime("%Y-%m-%d")}')

        limit = 500

        while current_dt >= end_dt and limit >= 0:
            partition = current_dt.strftime("%Y-%m-%d")
            print(f"Processing partition {partition}")

            plays_to_update = [
                play
                async for play in table.query(
                    key_condition=(
                        HashKey("pk", f"STATION#{station_id}#PLAYS#{partition}")
                        & RangeKey("sk").begins_with("PLAY#")
                    ),
                    filter_expression=(
                        F("gsi1pk").begins_with("STATION#") | F("gsi1sk").exists()
                    ),
                    projection=F("pk") & F("sk") & F("track_id"),
                )
            ]

            current_dt -= timedelta(days=1)
            limit -= 1

            print("building transaction items")

            transaction_items = [
                Update(
                    table=table_name,
                    key={"pk": play["pk"], "sk": play["sk"]},
                    expression=(
                        F("gsi1pk").set(f"TRACK#{play['track_id']}")
                        & F("gsi1sk").remove()
                    ),
                )
                for play in plays_to_update
            ]

            print(f"got {len(transaction_items)} to update")

            for chunk in batched(transaction_items, 100):
                print(f"committing a chunk of {len(chunk)}...")
                await client.transact_write_items(chunk)

    print("done")


if __name__ == "__main__":
    asyncio.run(main())

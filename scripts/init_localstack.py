#!/usr/bin/env python3

import boto3
import ulid
from datetime import datetime, UTC, timedelta

TABLE_NAME = "radiojournal-local"


def create_play(
    dynamodb,
    dt: datetime,
    station_id: str,
    track_id: str,
    title: str,
    artist: str,
    first_play: bool = False,
) -> str:
    play_id = ulid.from_timestamp(dt).str
    timestamp = dt.isoformat().replace("+00:00", "Z")

    dynamodb.put_item(
        TableName=TABLE_NAME,
        Item={
            "pk": {"S": f"STATION#{station_id}#PLAYS#{dt.strftime('%Y-%m-%d')}"},
            "sk": {"S": f"PLAY#{play_id}"},
            "gsi1pk": {"S": f"STATION#{station_id}#TRACK#{track_id}"},
            "gsi1sk": {"S": f"PLAY#{play_id}"},
            "id": {"S": play_id},
            "track_id": {"S": track_id},
            "created_ts": {"S": timestamp},
            "updated_ts": {"S": timestamp},
        },
    )

    dynamodb.update_item(
        TableName=TABLE_NAME,
        Key={
            "pk": {"S": f"STATION#{station_id}#TRACKS"},
            "sk": {"S": f"TRACK#{track_id}"},
        },
        UpdateExpression="SET latest_play_id = :val, play_count = play_count + :inc, updated_ts = :ts",
        ExpressionAttributeValues={
            ":val": {"S": play_id},
            ":ts": {"S": timestamp},
            ":inc": {"N": "1"},
        },
    )

    dynamodb.update_item(
        TableName=TABLE_NAME,
        Key={
            "pk": {"S": "STATIONS"},
            "sk": {"S": f"STATION#{station_id}"},
        },
        UpdateExpression="SET latest_play = :latest_play, play_count = play_count + :inc, updated_ts = :ts",
        ExpressionAttributeValues={
            ":latest_play": {
                "M": {
                    "id": {"S": play_id},
                    "track_id": {"S": track_id},
                    "artist": {"S": artist},
                    "title": {"S": title},
                }
            },
            ":inc": {"N": "1"},
            ":ts": {"S": timestamp},
        },
    )

    if first_play:
        dynamodb.update_item(
            TableName=TABLE_NAME,
            Key={
                "pk": {"S": "STATIONS"},
                "sk": {"S": f"STATION#{station_id}"},
            },
            UpdateExpression="SET first_play_id = :play_id, updated_ts = :ts",
            ExpressionAttributeValues={
                ":play_id": {"S": play_id},
                ":ts": {"S": timestamp},
            },
        )

    return play_id


def create_track(
    dynamodb,
    dt: datetime,
    station_id: str,
    artist: str,
    title: str,
    is_song: bool,
) -> str:
    track_id = ulid.from_timestamp(dt).str
    timestamp = dt.isoformat().replace("+00:00", "Z")

    dynamodb.put_item(
        TableName=TABLE_NAME,
        Item={
            "pk": {"S": f"STATION#{station_id}#TRACKS"},
            "sk": {"S": f"TRACK#{track_id}"},
            "id": {"S": track_id},
            "title": {"S": title},
            "artist": {"S": artist},
            "is_song": {"BOOL": is_song},
            "play_count": {"N": "0"},
            "latest_play_id": {"NULL": True},
            "created_ts": {"S": timestamp},
            "updated_ts": {"S": timestamp},
        },
    )

    dynamodb.put_item(
        TableName=TABLE_NAME,
        Item={
            "pk": {"S": f"STATION#{station_id}#ARTIST#{artist}"},
            "sk": {"S": f"TITLE#{title}"},
            "track_id": {"S": track_id},
        },
    )

    dynamodb.update_item(
        TableName=TABLE_NAME,
        Key={
            "pk": {"S": "STATIONS"},
            "sk": {"S": f"STATION#{station_id}"},
        },
        UpdateExpression="SET track_count = track_count + :inc, updated_ts = :ts",
        ExpressionAttributeValues={
            ":inc": {"N": "1"},
            ":ts": {"S": timestamp},
        },
    )

    return track_id


def create_station(
    dynamodb,
    dt: datetime,
    station_name: str,
    fetcher: str | None = None,
    fetcher_station: str | None = None,
) -> str:
    station_id = ulid.from_timestamp(dt).str
    timestamp = dt.isoformat().replace("+00:00", "Z")

    fetcher_obj = {"NULL": True}
    if fetcher:
        fetcher_obj = {"M": {"id": {"S": fetcher}}}
        if fetcher_station:
            fetcher_obj["M"]["station"] = {"S": fetcher_station}

    dynamodb.put_item(
        TableName=TABLE_NAME,
        Item={
            "pk": {"S": "STATIONS"},
            "sk": {"S": f"STATION#{station_id}"},
            "id": {"S": station_id},
            "name": {"S": station_name},
            "fetcher": fetcher_obj,
            "first_play_id": {"NULL": True},
            "latest_play": {"NULL": True},
            "track_count": {"N": "0"},
            "play_count": {"N": "0"},
            "created_ts": {"S": timestamp},
            "updated_ts": {"S": timestamp},
        },
    )

    return station_id


if __name__ == "__main__":
    dynamodb = boto3.client(
        "dynamodb",
        region_name="ap-southeast-1",
        endpoint_url="http://localhost:4566",
        aws_access_key_id="local",
        aws_secret_access_key="local",
    )

    try:
        dynamodb.delete_table(TableName=TABLE_NAME)
    except Exception:
        print("Skipping delete, table does not exist")

    dynamodb.create_table(
        TableName=TABLE_NAME,
        AttributeDefinitions=[
            {"AttributeName": "pk", "AttributeType": "S"},
            {"AttributeName": "sk", "AttributeType": "S"},
            {"AttributeName": "gsi1pk", "AttributeType": "S"},
            {"AttributeName": "gsi1sk", "AttributeType": "S"},
        ],
        KeySchema=[
            {"AttributeName": "pk", "KeyType": "HASH"},
            {"AttributeName": "sk", "KeyType": "RANGE"},
        ],
        GlobalSecondaryIndexes=[
            {
                "IndexName": "gsi1",
                "KeySchema": [
                    {"AttributeName": "gsi1pk", "KeyType": "HASH"},
                    {"AttributeName": "gsi1sk", "KeyType": "RANGE"},
                ],
                "Projection": {
                    "ProjectionType": "ALL",
                },
            },
        ],
        BillingMode="PAY_PER_REQUEST",
    )

    # create mock stations
    dt = datetime.now(tz=UTC)
    station_1 = create_station(
        dynamodb,
        dt,
        station_name="coolism",
        fetcher="coolism",
    )

    track_1 = create_track(
        dynamodb,
        dt + timedelta(minutes=3),
        station_1,
        artist="very cool artist",
        title="test title",
        is_song=True,
    )

    create_play(
        dynamodb,
        dt + timedelta(minutes=3),
        station_1,
        track_1,
        artist="very cool artist",
        title="test title",
        first_play=True,
    )

    track_2 = create_track(
        dynamodb,
        dt + timedelta(minutes=6),
        station_1,
        artist="soso artist",
        title="another test song",
        is_song=True,
    )

    create_play(
        dynamodb,
        dt + timedelta(minutes=6),
        station_1,
        track_2,
        artist="soso artist",
        title="another test song",
    )

    track_3 = create_track(
        dynamodb,
        dt + timedelta(minutes=9),
        station_1,
        artist="station jingle",
        title="not a song",
        is_song=False,
    )

    create_play(
        dynamodb,
        dt + timedelta(minutes=9),
        station_1,
        track_3,
        artist="station jingle",
        title="not a song",
    )

    create_play(
        dynamodb,
        dt + timedelta(minutes=10),
        station_1,
        track_1,
        artist="very cool artist",
        title="test title",
        # TODO: don't pass artist and title into create_play
    )

    dt = datetime.now(tz=UTC)
    station_2 = create_station(
        dynamodb,
        dt,
        station_name="efm",
        fetcher="atime",
        fetcher_station="efm",
    )

    dt = datetime.now(tz=UTC)
    station_3 = create_station(
        dynamodb,
        dt,
        station_name="greenwave",
        fetcher="atime",
        fetcher_station="greenwave",
    )

    dt = datetime.now(tz=UTC)
    station_4 = create_station(
        dynamodb,
        dt,
        station_name="chill",
        fetcher="atime",
        fetcher_station="chill",
    )

    print("Done")

#!/usr/bin/env python3

from xml.etree import ElementTree
import json
import urllib.request
from pathlib import Path

SOURCE_URL = "https://raw.githubusercontent.com/unicode-org/cldr/main/common/supplemental/windowsZones.xml"
OUTPUT_FILE = (
    Path(__file__).parent / ".." / "frontend/src/lib/data" / "windowsZones.json"
)


def fetch_xml() -> str:
    return urllib.request.urlopen(SOURCE_URL).read()


def process_full(zones):
    results = []
    for zone in zones:
        if (
            zone.tag == ElementTree.Comment
            and zone.text is not None
            and zone.text.strip().startswith("(UTC")
        ):
            results.append(
                {
                    "label": zone.text.strip(),
                    "territories": [],
                }
            )
            continue

        if len(results) > 0 and zone.tag == "mapZone":
            item = results[-1]
            if not "name" in item:
                item["name"] = zone.attrib["other"]
            else:
                assert item["name"] == zone.attrib["other"]

            item["territories"].append(
                {
                    "zoneinfo_ids": zone.attrib["type"].split(),
                    "name": zone.attrib["territory"],
                }
            )

    return results


def strip_minimal(zones):
    results = []
    for zone in zones:
        assert zone["territories"][0]["name"] == "001"
        results.append(
            {
                "name": zone["name"],
                "label": zone["label"].split(") ", maxsplit=1)[-1],
                "id": zone["territories"][0]["zoneinfo_ids"][0],
            }
        )

    return results


if __name__ == "__main__":
    print("Fetching windowsZones XML")
    resp = fetch_xml()

    parser = ElementTree.XMLParser(target=ElementTree.TreeBuilder(insert_comments=True))
    root = ElementTree.fromstring(resp, parser=parser)

    zones = root.find("./windowsZones/mapTimezones")
    if zones is None:
        raise Exception("cannot find ./windowsZones/mapTimezones in xml")

    print("Parsing timezones")
    full_results = process_full(zones)
    results = strip_minimal(full_results)

    print(f"Writing to file {OUTPUT_FILE.resolve()}")
    with Path.open(OUTPUT_FILE, "w") as f:
        f.write(json.dumps(results, indent=2))
        f.write("\n")

    print("Done")

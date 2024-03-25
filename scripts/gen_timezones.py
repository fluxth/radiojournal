#!/usr/bin/env python3

from xml.etree import ElementTree
import json

import requests


URL = "https://raw.githubusercontent.com/unicode-org/cldr/main/common/supplemental/windowsZones.xml"


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
                "label": zone["label"],
                "id": zone["territories"][0]["zoneinfo_ids"][0],
            }
        )

    return results


if __name__ == "__main__":
    resp = requests.get(URL)

    parser = ElementTree.XMLParser(target=ElementTree.TreeBuilder(insert_comments=True))
    root = ElementTree.fromstring(resp.text, parser=parser)

    zones = root.find("./windowsZones/mapTimezones")
    if zones is None:
        raise Exception("cannot find ./windowsZones/mapTimezones in xml")

    results = process_full(zones)
    results = strip_minimal(results)

    print(json.dumps(results, indent=2))

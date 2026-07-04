import argparse
import hashlib
import json
import sys
from pathlib import Path


IMAGE_COLLECTIONS = ("proof", "proofRepeat", "debug", "morph")


class VerificationError(Exception):
    pass


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise VerificationError(message)


def candidate_images(stats: dict) -> list[tuple[str, str]]:
    images: list[tuple[str, str]] = []
    seen: set[str] = set()
    for collection in IMAGE_COLLECTIONS:
        for item in stats[collection]:
            image = item["image"]
            file_name = image["file"]
            require(file_name not in seen, f"duplicate candidate image: {file_name}")
            seen.add(file_name)
            images.append((file_name, image["sha256"]))
    return images


def verify(candidate_stats_path: Path, capture_root: Path) -> list[tuple[str, str]]:
    rows: list[tuple[str, str]] = []
    stats = read_json(candidate_stats_path)

    html_path = capture_root / "mf_proto_live_steel.html"
    require(html_path.exists(), f"captured HTML missing: {html_path}")
    expected_html = stats.get("htmlSha256")
    require(expected_html, "candidate htmlSha256 missing")
    actual_html = sha256_file(html_path)
    require(actual_html == expected_html, f"HTML drift: {actual_html} != {expected_html}")
    rows.append(("candidate_html_sha256", "PASS"))

    checked = 0
    for file_name, expected_sha in candidate_images(stats):
        image_path = capture_root / file_name
        require(image_path.exists(), f"captured PNG missing: {file_name}")
        actual_sha = sha256_file(image_path)
        require(actual_sha == expected_sha, f"PNG drift: {file_name}: {actual_sha} != {expected_sha}")
        checked += 1
    rows.append((f"candidate_png_hashes:{checked}", "PASS"))
    return rows


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--candidate-stats", required=True)
    parser.add_argument("--capture-root", required=True)
    args = parser.parse_args()

    try:
        rows = verify(Path(args.candidate_stats).resolve(), Path(args.capture_root).resolve())
    except VerificationError as exc:
        print(f"FAIL: {exc}")
        sys.exit(1)

    width = max(len(name) for name, _status in rows)
    for name, status in rows:
        print(f"{name.ljust(width)}  {status}")


if __name__ == "__main__":
    main()

import unittest
from pathlib import Path
import sys

SCRIPT_DIR = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIR))

from common import repo_root  # noqa: E402


class SeededOverviewScriptTests(unittest.TestCase):
    def test_repo_root_points_at_tm_repository(self):
        root = repo_root()
        self.assertEqual(root.name, "tm")
        self.assertTrue((root / "Cargo.toml").exists(), "repo root should contain Cargo.toml")
        self.assertTrue((root / "crates" / "ui").exists(), "repo root should expose crates/ui")


if __name__ == "__main__":
    unittest.main()

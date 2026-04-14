import json
import subprocess
import sys
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path


sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))

import run_seeded_overview_check


class SeededOverviewScriptTests(unittest.TestCase):
    def test_seeded_overview_script_supports_dry_run_and_emits_report_shape(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            report_path = Path(tmpdir) / "overview-report.json"
            script = (
                Path(__file__).resolve().parents[1]
                / "scripts"
                / "run_seeded_overview_check.py"
            )

            result = subprocess.run(
                [
                    sys.executable,
                    str(script),
                    "--dry-run",
                    "--report",
                    str(report_path),
                ],
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(report_path.exists(), "script should write a report json")

            payload = json.loads(report_path.read_text())
            self.assertIn(payload["status"], {"PASS", "FAIL", "BLOCKED"})
            self.assertEqual(payload["mode"], "seeded-overview")
            self.assertIn("artifacts", payload)
            self.assertIn("checks", payload)
            self.assertIn("environment", payload)

    def test_seeded_overview_fixture_matches_current_day_range(self):
        now = datetime(2026, 4, 14, 12, 0, 0, tzinfo=timezone.utc)

        fixture = run_seeded_overview_check.seeded_overview_fixture(now)

        self.assertEqual(fixture["range"]["started_at"], "2026-04-14T00:00:00Z")
        self.assertEqual(fixture["range"]["ended_at"], "2026-04-14T23:59:59Z")
        self.assertEqual(fixture["rows"][0][3], "2026-04-14T09:00:00Z")
        self.assertEqual(fixture["rows"][1][3], "2026-04-14T10:00:00Z")


if __name__ == "__main__":
    unittest.main()

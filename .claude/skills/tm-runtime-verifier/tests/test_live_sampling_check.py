import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


class LiveSamplingScriptTests(unittest.TestCase):
    def test_live_sampling_script_supports_dry_run_and_emits_report_shape(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            report_path = Path(tmpdir) / "live-report.json"
            script = Path(__file__).resolve().parents[1] / "scripts" / "run_live_sampling_check.py"

            result = subprocess.run(
                [sys.executable, str(script), "--dry-run", "--report", str(report_path)],
                capture_output=True,
                text=True,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(report_path.exists(), "script should write a report json")

            payload = json.loads(report_path.read_text())
            self.assertIn(payload["status"], {"PASS", "FAIL", "BLOCKED"})
            self.assertEqual(payload["mode"], "live-sampling")
            self.assertIn("checks", payload)
            self.assertIn("artifacts", payload)
            self.assertIn("environment", payload)


if __name__ == "__main__":
    unittest.main()

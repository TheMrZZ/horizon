#!/usr/bin/env python3
"""Live generic mob air with water breathing, dry refill and underwater controls."""
from pathlib import Path
import subprocess
import sys

if __name__ == '__main__':
    root = Path(__file__).resolve().parents[1]
    raise SystemExit(subprocess.run([
        sys.executable, str(root / 'tools/test_command_exec_parity.py'),
        '--only', 'mob water breathing', '--work-dir', str(root / '.local/mob-water-breathing-parity'),
        *sys.argv[1:],
    ], cwd=root).returncode)

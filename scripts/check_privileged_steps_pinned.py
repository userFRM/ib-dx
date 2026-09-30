"""Every step of a job holding a write permission names its action by commit.

A workflow that grants itself a write permission acts as the repository. An
action it resolves by moving tag is a version nobody reviewed: the tag can be
repointed between the reading of the workflow and the running of it, and what
arrives spends the permission the job holds. The publishing jobs of the
release workflow were pinned by commit for exactly this reason; this is the
same rule kept true everywhere a job holds pages, id-token or contents write.

A pinned step names the full commit SHA and carries the tag it was resolved
from in a comment beside it, so a reader can see both what runs and what it
was when it was chosen. A reusable workflow of this repository is a file in
the tree being run, not a moving tag, and is not a finding.

Exits non-zero on its own findings.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
WORKFLOWS = ROOT / ".github/workflows"
PERM = re.compile(r"^\s+([a-z-]+):\s*(read|write|none)\s*$")
JOB = re.compile(r"^ {2}([A-Za-z0-9_-]+):\s*$")
USES = re.compile(r"^\s*(?:-\s+)?uses:\s*(\S+)\s*(?:#\s*(\S.*?))?\s*$")
SHA = re.compile(r"^[0-9a-f]{40}$")


def perms_under(lines, i, indent):
    """The key: value pairs of the block opened at lines[i]."""
    perms = {}
    for line in lines[i + 1:]:
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        here = len(line) - len(line.lstrip())
        if here <= indent:
            break
        m = PERM.match(line)
        if m:
            perms[m.group(1)] = m.group(2)
    return perms


def audit(path):
    lines = path.read_text().splitlines()

    defaults = {}
    for i, line in enumerate(lines):
        if line == "permissions:":
            defaults = perms_under(lines, i, 0)
            break

    in_jobs = False
    starts = []
    for i, line in enumerate(lines):
        if line == "jobs:":
            in_jobs = True
            continue
        if in_jobs:
            if line[:1].strip() and not line.startswith(" "):
                break  # the next top-level key ends the block
            if JOB.match(line):
                starts.append((i, JOB.match(line).group(1)))

    problems = []
    for n, (start, job) in enumerate(starts):
        end = starts[n + 1][0] if n + 1 < len(starts) else len(lines)
        block = lines[start:end]

        perms = defaults
        for j, line in enumerate(block):
            if line.rstrip() == "    permissions:":
                perms = perms_under(block, j, 4)
                break
        if "write" not in perms.values():
            continue

        for j, line in enumerate(block):
            m = USES.match(line)
            if not m:
                continue
            ref, comment = m.groups()
            if ref.startswith("./") or ref.startswith(".github/"):
                continue  # a reusable workflow is a file in this repository
            where = f"{path.name}:{start + j + 1}"
            action, _, version = ref.partition("@")
            holding = ", ".join(k for k, v in perms.items() if v == "write")
            if not SHA.match(version):
                problems.append(
                    f"{where}: {job} holds {holding} write and resolves "
                    f"{action} by tag, not by commit SHA"
                )
            elif not comment:
                problems.append(
                    f"{where}: {job} holds {holding} write and pins {action} "
                    "without recording the tag the SHA was resolved from"
                )
    return problems


problems = [p for f in sorted(WORKFLOWS.glob("*.yml")) for p in audit(f)]
if problems:
    print("\n".join(problems))
    print(f"\n{len(problems)} step(s) of a privileged job are not pinned by commit.")
    sys.exit(1)

print("every step of every job holding a write permission is pinned by commit")

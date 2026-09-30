"""A link on the generated reference pages leads to something.

A doc comment links to a neighbouring item by its path, which rustdoc resolves
and Markdown does not. Copied onto the book's pages as written, one became a
link to a relative path that is not there, others printed their brackets, and
a line naming a link's target was joined onto the end of its paragraph.
"""

import importlib.util
import pathlib
import re

from ibkr_dx import EWrapper

ROOT = pathlib.Path(__file__).resolve().parents[2]


def _generator():
    spec = importlib.util.spec_from_file_location(
        "gen_api_docs", ROOT / "scripts" / "gen_api_docs.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


#: A path rustdoc resolves and a book page cannot: in parentheses as a link's
#: target, in brackets on its own, or named as a target after a colon.
UNRESOLVED = re.compile(
    r"\]\((?:Self|crate|[A-Z]\w*)::[^)]*\)"
    r"|\[`(?:Self|crate|[A-Z]\w*)[^`\]]*`\](?!\()"
    r"|\]:\s*(?:Self|crate)::"
)


def test_each_spelling_of_a_path_link_keeps_its_words():
    plain = _generator().plain_intra_doc_links
    assert plain("The same clock [`current_time`](Self::current_time) reports.") == (
        "The same clock `current_time` reports."
    )
    assert plain("Read it with [`Self::scanned_strategies`] under it.") == (
        "Read it with `Self::scanned_strategies` under it."
    )
    assert plain(
        "[`Wrapper::tick_generic`] carries it. "
        "[`Wrapper::tick_generic`]: crate::api::Wrapper::tick_generic"
    ) == "`Wrapper::tick_generic` carries it."
    assert plain("See [the book](https://example.com/x).") == "See [the book](https://example.com/x)."


def test_the_generated_pages_link_nowhere_missing():
    generator = _generator()
    version = generator.version()
    for page, _ in (generator.generate_rust_md(version), generator.generate_python_md(version)):
        left = UNRESOLVED.findall(page)
        assert not left, f"links a book page cannot follow: {left[:5]}"


def test_the_generator_sees_every_callback_the_class_carries():
    """The page documents what the parser reads off the source.

    A lint allowance standing between `#[pymethods]` and its `impl` ended the
    block match, and the generated page lost every callback while the class
    itself carried them all — the runtime surface intact, the documentation
    of it empty.
    """
    parsed = {
        method["name"]
        for method in _generator().parse_pymethods(
            ROOT / "src" / "python" / "compat" / "wrapper.rs"
        )
    }
    carried = {
        name for name in dir(EWrapper)
        if name.islower() and not name.startswith("_")
        # The protobuf family's stubs are installed from Python, not parsed
        # out of this file; the Limits page documents them.
        and not name.endswith("_proto_buf")
    }
    assert carried, "the class carries no callbacks to see"
    assert carried <= parsed, f"callbacks the parser cannot see: {sorted(carried - parsed)[:5]}"

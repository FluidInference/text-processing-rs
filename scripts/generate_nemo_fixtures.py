#!/usr/bin/env python3
"""Regenerate the FST engine's ITN goldens from NeMo's own output.

For every NeMo ITN test file (tests/nemo_text_processing/<lang>/
data_inverse_text_normalization/test_cases_*.txt in the NeMo checkout), runs
InverseNormalizer(lang, input_case="cased") on each spoken input, then NeMo's
own punctuation post-processing (the Moses detokenizer and post_process_punct,
as NeMo's TN applies them), and writes `spoken~NeMo output` to
tests/fixtures/itn/<lang>/. The goldens are NeMo's actual output, not the
upstream expected column, so the engine is held to byte-exact NeMo behaviour.

  PYTHONPATH=/path/to/NeMo-text-processing python scripts/generate_nemo_fixtures.py --lang en \
      --nemo /path/to/NeMo-text-processing
"""

import argparse
import glob
import os

PINNED = "1f1263579fe57ba7ed783cad3dddee710fcc5064"


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--lang", required=True)
    parser.add_argument("--nemo", required=True, help="NeMo-text-processing checkout")
    parser.add_argument("--out", default=None)
    parser.add_argument("--cache-dir", default=None)
    args = parser.parse_args()

    from nemo_text_processing.inverse_text_normalization.inverse_normalize import InverseNormalizer
    from nemo_text_processing.text_normalization.data_loader_utils import post_process_punct
    from sacremoses import MosesDetokenizer

    normalizer = InverseNormalizer(input_case="cased", lang=args.lang, cache_dir=args.cache_dir)
    moses = MosesDetokenizer(lang=args.lang)
    src = os.path.join(args.nemo, "tests", "nemo_text_processing", args.lang, "data_inverse_text_normalization")
    out_dir = args.out or os.path.join(os.path.dirname(__file__), "..", "tests", "fixtures", "itn", args.lang)
    os.makedirs(out_dir, exist_ok=True)
    for path in sorted(glob.glob(os.path.join(src, "test_cases_*.txt"))):
        rows, skipped = [], []
        with open(path, encoding="utf-8") as f:
            for line in f:
                line = line.rstrip("\n")
                if not line.strip() or "~" not in line:
                    continue
                spoken = line.split("~", 1)[0]
                written = normalizer.inverse_normalize(spoken, verbose=False)
                written = moses.detokenize([written], unescape=False)
                written = post_process_punct(input=spoken, normalized_text=written)
                # NeMo's token parser sometimes leaks raw tag syntax into its output
                # (zh ordinals: `第1億" } } tokens { cardinal ...`). That is a NeMo bug,
                # not behaviour to reproduce, so such cases are left out.
                if "tokens {" in written:
                    skipped.append(spoken)
                    continue
                rows.append((spoken, written))
        with open(os.path.join(out_dir, os.path.basename(path)), "w", encoding="utf-8") as f:
            f.write(
                f"# NeMo ITN output (lang={args.lang}, input_case=cased, punctuation post-processed),"
                " regenerated from NeMo-text-processing\n"
            )
            f.write(f"# pinned commit {PINNED}. Format: spoken~written.\n")
            for spoken in skipped:
                f.write(f"# left out, NeMo's output leaks tag syntax: {spoken}\n")
            for spoken, written in rows:
                f.write(f"{spoken}~{written}\n")


if __name__ == "__main__":
    main()

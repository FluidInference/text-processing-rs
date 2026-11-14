#!/usr/bin/env python3
"""Export NeMo-text-processing's compiled grammars for the `fst-engine` feature.

Runs NeMo's own grammar builders (pynini) and writes the FSTs the Rust engine
loads, gzipped, into grammars/<lang>/:

  TN:  classify.fst.gz, verbalize.fst.gz, postprocess.fst.gz (en, hi only)
  ITN: itn_classify.fst.gz, itn_verbalize.fst.gz

Needs pynini (2.1.7) and a NeMo-text-processing checkout on PYTHONPATH at the
pinned commit (NEMO_REF in .github/workflows/parity.yml). The FSTs are written
untouched: calling properties() before write() changes the header.

  PYTHONPATH=/path/to/NeMo-text-processing python scripts/export_nemo_grammars.py --mode itn --lang en
"""

import argparse
import gzip
import os
import shutil
import tempfile


def write_gz(fst, path):
    with tempfile.TemporaryDirectory() as tmp:
        raw = os.path.join(tmp, "g.fst")
        fst.write(raw)
        with open(raw, "rb") as src, open(path, "wb") as dst:
            # mtime=0 and no filename keep the output reproducible (gzip -n).
            with gzip.GzipFile(filename="", mode="wb", fileobj=dst, compresslevel=6, mtime=0) as gz:
                shutil.copyfileobj(src, gz)


def export_tn(lang, out_dir, cache_dir):
    from nemo_text_processing.text_normalization.normalize import Normalizer

    normalizer = Normalizer(input_case="cased", lang=lang, deterministic=True, cache_dir=cache_dir)
    write_gz(normalizer.tagger.fst, os.path.join(out_dir, "classify.fst.gz"))
    write_gz(normalizer.verbalizer.fst, os.path.join(out_dir, "verbalize.fst.gz"))
    post = getattr(normalizer, "post_processor", None)
    if post is not None:
        write_gz(post.fst, os.path.join(out_dir, "postprocess.fst.gz"))


def export_itn(lang, out_dir, cache_dir):
    from nemo_text_processing.inverse_text_normalization.inverse_normalize import InverseNormalizer

    normalizer = InverseNormalizer(input_case="lower_cased", lang=lang, cache_dir=cache_dir)
    write_gz(normalizer.tagger.fst, os.path.join(out_dir, "itn_classify.fst.gz"))
    write_gz(normalizer.verbalizer.fst, os.path.join(out_dir, "itn_verbalize.fst.gz"))


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--mode", choices=["tn", "itn"], required=True)
    parser.add_argument("--lang", required=True)
    parser.add_argument("--out", default=None, help="output directory (default grammars/<lang>)")
    parser.add_argument("--cache-dir", default=None, help="NeMo .far cache directory")
    args = parser.parse_args()
    out_dir = args.out or os.path.join(os.path.dirname(__file__), "..", "grammars", args.lang)
    os.makedirs(out_dir, exist_ok=True)
    (export_tn if args.mode == "tn" else export_itn)(args.lang, out_dir, args.cache_dir)


if __name__ == "__main__":
    main()

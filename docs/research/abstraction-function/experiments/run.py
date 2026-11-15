"""Entry point. Run with the interpreter in isolated mode, from any directory:

    python -I run.py explore --rq <rq checkout> --out <dir> [--bound N] [--scenarios a,b]
    python -I run.py observe --rq <rq checkout> --graphs <dir>
"""

from __future__ import annotations

import argparse
import logging
import pathlib
import pickle
import sys

HERE = pathlib.Path(__file__).resolve().parent


def _load_rq(rq_path: str) -> str:
    logging.disable(logging.CRITICAL)
    sys.path.insert(0, str(HERE))
    sys.path.insert(0, str(pathlib.Path(rq_path).resolve()))
    import rq

    from afexp import harness

    harness.install_patches()
    harness.freeze_rq_clock()
    harness.deterministic_ids()
    import afjobs

    afjobs.ON_START.append(harness.job_started)
    return str(getattr(rq, '__version__', getattr(rq, 'VERSION', '?')))


def main() -> None:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest='command', required=True)
    explore = sub.add_parser('explore')
    explore.add_argument('--rq', required=True)
    explore.add_argument('--out', required=True)
    explore.add_argument('--bound', type=int, default=2)
    explore.add_argument('--scenarios', default='')
    observe = sub.add_parser('observe')
    observe.add_argument('--rq', required=True)
    observe.add_argument('--graphs', required=True)
    observe.add_argument('--purity', choices=['semantic', 'syntactic'], default='semantic')
    probe = sub.add_parser('probe')
    probe.add_argument('--rq', required=True)
    probe.add_argument('--graphs', required=True)
    probe.add_argument('--depth', type=int, default=2)
    evaluate = sub.add_parser('evaluate')
    evaluate.add_argument('--graphs', required=True)
    evaluate.add_argument('--out', required=True)
    evaluate.add_argument('--spec-from', default=None, help='graph dir whose training runs derive our approach')
    evaluate.add_argument('--llm', default=None, help='directory of stored LLM answers')
    compare = sub.add_parser('compare')
    compare.add_argument('--base', required=True)
    compare.add_argument('--mutants', required=True)
    compare.add_argument('--versions', nargs='*', default=[])
    compare.add_argument('--llm', default=None)
    compare.add_argument('--out', required=True)
    args = parser.parse_args()

    if args.command == 'compare':
        sys.path.insert(0, str(HERE))
        from afexp import compare as cmp

        cmp.run(
            pathlib.Path(args.base),
            pathlib.Path(args.mutants),
            [pathlib.Path(v) for v in args.versions],
            pathlib.Path(args.llm) if args.llm else None,
            pathlib.Path(args.out),
        )
        return
    if args.command == 'evaluate':
        sys.path.insert(0, str(HERE))
        from afexp import evaluate as ev

        ev.run_evaluation(
            pathlib.Path(args.graphs),
            pathlib.Path(args.out),
            pathlib.Path(args.spec_from) if args.spec_from else None,
            pathlib.Path(args.llm) if args.llm else None,
        )
        return
    version = _load_rq(args.rq)
    if args.command == 'explore':
        from afexp import scenarios
        from afexp.explore import explore as run_explore

        out = pathlib.Path(args.out)
        out.mkdir(parents=True, exist_ok=True)
        built = scenarios.build()
        names = args.scenarios.split(',') if args.scenarios else scenarios.ALL
        for name in names:
            graph = run_explore(name, built[name], args.bound)
            graph['rq_version'] = version
            graph['rq_path'] = str(args.rq)
            errors = sum(1 for r in graph['runs'] if r['error'])
            print(f'{name}: rq {version}, bound {args.bound}, {graph["meta"]}, errors {errors}', flush=True)
            with open(out / f'{name}.b{args.bound}.pkl', 'wb') as fh:
                pickle.dump(graph, fh)
    elif args.command == 'probe':
        from afexp import probes

        probes.probe_dir(pathlib.Path(args.graphs), args.depth)
    else:
        from afexp import observers

        observers.observe_dir(pathlib.Path(args.graphs), args.purity)


if __name__ == '__main__':
    main()

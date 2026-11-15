  work-ok: {'runs': 18, 'unserialisable': 0, 'invariant_broken': 0, 'anomalies': 0, 'sequential_runs_breaking_invariant': '0/2'}
  fail-retry: {'runs': 336, 'unserialisable': 0, 'invariant_broken': 0, 'anomalies': 0, 'sequential_runs_breaking_invariant': '0/4'}
  requeue-vs-work: {'runs': 40, 'unserialisable': 0, 'invariant_broken': 0, 'anomalies': 0, 'sequential_runs_breaking_invariant': '0/2'}
  dependency: {'runs': 423, 'unserialisable': 20, 'invariant_broken': 20, 'anomalies': 20, 'sequential_runs_breaking_invariant': '0/4'}
  cancel-vs-work: {'runs': 410, 'unserialisable': 137, 'invariant_broken': 331, 'anomalies': 341, 'sequential_runs_breaking_invariant': '1/3'}
  cleanup-vs-work: {'runs': 64, 'unserialisable': 28, 'invariant_broken': 54, 'anomalies': 54, 'sequential_runs_breaking_invariant': '1/3'}
  two-workers: {'runs': 360, 'unserialisable': 0, 'invariant_broken': 0, 'anomalies': 0, 'sequential_runs_breaking_invariant': '0/4'}
  delete-vs-work: {'runs': 446, 'unserialisable': 139, 'invariant_broken': 139, 'anomalies': 139, 'sequential_runs_breaking_invariant': '0/3'}
| AF | abs states | findings | P | R | F1 | held-out F1 | det | changes/op | homog | compl |
|---|---|---|---|---|---|---|---|---|---|---|
| identity | 688 | 602 | 0.307 | 0.792 | 0.443 | 0.554 | 0.941 | 3.56 | 1.0 | 0.662 |
| status-only | 50 | 13 | 0.556 | 0.162 | 0.251 | 0.262 | 0.967 | 1.18 | 0.775 | 1.0 |
| membership | 97 | 38 | 0.768 | 0.787 | 0.777 | 0.805 | 0.969 | 1.89 | 0.932 | 0.966 |
| reference | 98 | 45 | 0.76 | 0.787 | 0.773 | 0.8 | 0.969 | 1.89 | 1.0 | 1.0 |
| read-projection | 86 | 22 | 0.454 | 0.168 | 0.245 | 0.269 | 0.955 | 1.54 | 0.866 | 0.892 |
| behavioral-d1@2.12.0 | 44 | 14 | 0.554 | 0.148 | 0.234 | 0.259 | 0.88 | 1.04 | 0.621 | 0.974 |
| behavioral-d2@2.12.0 | 71 | 36 | 0.635 | 0.348 | 0.45 | 0.449 | 1.0 | 1.49 | 0.794 | 0.954 |
| observer-semantic@2.12.0 | 134 | 65 | 0.72 | 0.792 | 0.754 | 0.779 | 0.955 | 1.94 | 0.997 | 0.888 |
| observer-syntactic@2.12.0 | 121 | 48 | 0.693 | 0.563 | 0.622 | 0.638 | 0.955 | 1.88 | 0.966 | 0.9 |
| reference+doc-invariant | 98 | 59 | 0.798 | 0.982 | 0.88 | 0.913 | 0.969 | 1.89 | 1.0 | 1.0 |
| llm:opus-code-1 | 502 | 392 | 0.315 | 0.792 | 0.451 | 0.561 | 0.939 | 2.74 | 1.0 | 0.697 |
| llm:opus-code-1+inv | 502 | 397 | 0.354 | 0.944 | 0.515 | 0.638 | 0.939 | 2.74 | 1.0 | 0.697 |
| llm:opus-code-1:inv-only | 18 | 2 | 0.962 | 0.37 | 0.535 | 0.549 | 0.867 | 0.33 | 0.259 | 1.0 |
| llm:opus-code-2 | 212 | 141 | 0.334 | 0.792 | 0.47 | 0.582 | 0.939 | 2.71 | 0.989 | 0.788 |
| llm:opus-code-2+inv | 212 | 160 | 0.384 | 0.982 | 0.552 | 0.678 | 0.939 | 2.71 | 0.989 | 0.788 |
| llm:opus-code-2:inv-only | 18 | 9 | 0.962 | 0.731 | 0.831 | 0.824 | 0.867 | 0.33 | 0.259 | 1.0 |
| llm:opus-docs-1 | 457 | 345 | 0.315 | 0.792 | 0.451 | 0.561 | 0.939 | 2.7 | 1.0 | 0.702 |
| llm:opus-docs-1+inv | 457 | 351 | 0.354 | 0.944 | 0.515 | 0.638 | 0.939 | 2.7 | 1.0 | 0.702 |
| llm:opus-docs-1:inv-only | 18 | 3 | 0.963 | 0.375 | 0.54 | 0.555 | 0.867 | 0.33 | 0.259 | 1.0 |
| llm:opus-docs-2 | 502 | 392 | 0.315 | 0.792 | 0.451 | 0.561 | 0.939 | 2.74 | 1.0 | 0.697 |
| llm:opus-docs-2+inv | 502 | 410 | 0.363 | 0.982 | 0.53 | 0.655 | 0.939 | 2.74 | 1.0 | 0.697 |
| llm:opus-docs-2:inv-only | 18 | 6 | 0.981 | 0.731 | 0.838 | 0.831 | 0.867 | 0.33 | 0.259 | 1.0 |
| llm:opus-docs-3 | 497 | 385 | 0.315 | 0.792 | 0.451 | 0.561 | 0.939 | 2.73 | 0.998 | 0.699 |
| llm:opus-docs-3+inv | 497 | 405 | 0.355 | 0.982 | 0.522 | 0.642 | 0.939 | 2.73 | 0.998 | 0.699 |
| llm:opus-docs-3:inv-only | 18 | 13 | 0.81 | 0.731 | 0.769 | 0.759 | 0.867 | 0.33 | 0.259 | 1.0 |
| llm:sonnet-docs-1 | 214 | 144 | 0.329 | 0.792 | 0.465 | 0.574 | 0.938 | 2.67 | 0.999 | 0.807 |
| llm:sonnet-docs-1+inv | 214 | 154 | 0.361 | 0.912 | 0.517 | 0.635 | 0.938 | 2.67 | 0.999 | 0.807 |
| llm:sonnet-docs-1:inv-only | 18 | 6 | 1.0 | 0.543 | 0.704 | 0.69 | 0.867 | 0.33 | 0.259 | 1.0 |
| llm:sonnet-docs-2 | 223 | 145 | 0.329 | 0.792 | 0.465 | 0.574 | 0.941 | 2.7 | 0.999 | 0.797 |
| llm:sonnet-docs-2+inv | 223 | 163 | 0.361 | 0.982 | 0.528 | 0.642 | 0.941 | 2.7 | 0.999 | 0.797 |
| llm:sonnet-docs-2:inv-only | 18 | 11 | 0.81 | 0.731 | 0.769 | 0.759 | 0.867 | 0.33 | 0.259 | 1.0 |
| llm:sonnet-docs-3 | 219 | 148 | 0.329 | 0.792 | 0.465 | 0.574 | 0.939 | 2.73 | 0.999 | 0.8 |
| llm:sonnet-docs-3+inv | 219 | 172 | 0.355 | 0.982 | 0.521 | 0.642 | 0.939 | 2.73 | 0.999 | 0.8 |
| llm:sonnet-docs-3:inv-only | 18 | 15 | 0.651 | 0.978 | 0.782 | 0.912 | 0.867 | 0.33 | 0.259 | 1.0 |
| own-select-only | 54 | 13 | 0.833 | 0.162 | 0.272 | 0.284 | 1.0 | 1.16 | 0.801 | 0.942 |
| own | 54 | 22 | 0.938 | 0.709 | 0.808 | 0.806 | 1.0 | 1.16 | 0.801 | 0.942 |
| own@all-seq-select-only | 57 | 15 | 0.833 | 0.162 | 0.272 | 0.284 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq | 57 | 24 | 0.942 | 0.76 | 0.841 | 0.842 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent:opus-code-1 | 57 | 28 | 0.952 | 0.939 | 0.945 | 0.951 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-train:opus-code-1 | 57 | 28 | 0.952 | 0.939 | 0.945 | 0.951 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-all:opus-code-1 | 57 | 26 | 0.942 | 0.76 | 0.841 | 0.842 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent:opus-code-2 | 57 | 32 | 0.954 | 0.982 | 0.968 | 0.974 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-train:opus-code-2 | 57 | 32 | 0.954 | 0.982 | 0.968 | 0.974 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-all:opus-code-2 | 57 | 26 | 0.942 | 0.76 | 0.841 | 0.842 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent:opus-docs-1 | 57 | 29 | 0.953 | 0.944 | 0.948 | 0.954 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-train:opus-docs-1 | 57 | 29 | 0.953 | 0.944 | 0.948 | 0.954 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-all:opus-docs-1 | 57 | 27 | 0.942 | 0.765 | 0.845 | 0.845 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent:opus-docs-2 | 57 | 32 | 0.954 | 0.982 | 0.968 | 0.974 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-train:opus-docs-2 | 57 | 32 | 0.954 | 0.982 | 0.968 | 0.974 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-all:opus-docs-2 | 57 | 26 | 0.942 | 0.76 | 0.841 | 0.842 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent:opus-docs-3 | 57 | 33 | 0.841 | 0.982 | 0.906 | 0.909 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-train:opus-docs-3 | 57 | 33 | 0.841 | 0.982 | 0.906 | 0.909 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-all:opus-docs-3 | 57 | 27 | 0.942 | 0.76 | 0.841 | 0.842 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent:sonnet-docs-1 | 57 | 28 | 0.95 | 0.884 | 0.916 | 0.92 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-train:sonnet-docs-1 | 57 | 28 | 0.95 | 0.884 | 0.916 | 0.92 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-all:sonnet-docs-1 | 57 | 25 | 0.942 | 0.76 | 0.841 | 0.842 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent:sonnet-docs-2 | 57 | 33 | 0.841 | 0.982 | 0.906 | 0.909 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-train:sonnet-docs-2 | 57 | 33 | 0.841 | 0.982 | 0.906 | 0.909 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-all:sonnet-docs-2 | 57 | 27 | 0.942 | 0.76 | 0.841 | 0.842 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent:sonnet-docs-3 | 57 | 35 | 0.641 | 0.982 | 0.775 | 0.909 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-train:sonnet-docs-3 | 57 | 34 | 0.841 | 0.982 | 0.906 | 0.909 | 1.0 | 1.16 | 0.815 | 0.943 |
| own@all-seq+intent-houdini-all:sonnet-docs-3 | 57 | 29 | 0.942 | 0.76 | 0.841 | 0.842 | 1.0 | 1.16 | 0.815 | 0.943 |
| own-all-observables-select-only | 70 | 16 | 0.838 | 0.168 | 0.28 | 0.292 | 0.955 | 1.19 | 0.806 | 0.885 |
| own-all-observables | 70 | 27 | 0.941 | 0.753 | 0.837 | 0.837 | 0.955 | 1.19 | 0.806 | 0.885 |
| own+llm-select-only | 41 | 4 | 0.138 | 0.054 | 0.078 | 0.088 | 0.885 | 0.97 | 0.571 | 0.973 |
| own+llm | 41 | 18 | 0.568 | 0.899 | 0.696 | 0.744 | 0.885 | 0.97 | 0.571 | 0.973 |

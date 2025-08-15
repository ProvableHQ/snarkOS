window.BENCHMARK_DATA = {
  "lastUpdate": 1755228291848,
  "repoUrl": "https://github.com/ProvableHQ/snarkOS",
  "entries": {
    "snarkOS Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "kai@provable.com",
            "name": "Kai Mast",
            "username": "kaimast"
          },
          "committer": {
            "email": "kai@provable.com",
            "name": "Kai Mast",
            "username": "kaimast"
          },
          "distinct": true,
          "id": "c87c4ab38f437da02d076abb1074691a9f5aadde",
          "message": "ci: add sync benchmark",
          "timestamp": "2025-08-14T14:10:40-07:00",
          "tree_id": "c530b1f7d02663e6f3b95d4c33ca755206893e30",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/c87c4ab38f437da02d076abb1074691a9f5aadde"
        },
        "date": 1755207642550,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 4.34,
            "unit": "blocks/s",
            "extra": "total_wait=46s"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "kai@provable.com",
            "name": "Kai Mast",
            "username": "kaimast"
          },
          "committer": {
            "email": "kai@provable.com",
            "name": "Kai Mast",
            "username": "kaimast"
          },
          "distinct": true,
          "id": "8dee5ad57797adffb9efed571bf04b41da69aa90",
          "message": "ci: add sync benchmark",
          "timestamp": "2025-08-14T19:46:26-07:00",
          "tree_id": "a183239d41a40a7a99b85cf0ff48b3a9a0bfaa39",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/8dee5ad57797adffb9efed571bf04b41da69aa90"
        },
        "date": 1755228291523,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 4.44,
            "unit": "blocks/s",
            "extra": "total_wait=45s"
          },
          {
            "name": "cdn-sync",
            "value": 0.54,
            "unit": "blocks/s",
            "extra": "total_wait=183s"
          }
        ]
      }
    ]
  }
}
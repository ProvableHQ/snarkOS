window.BENCHMARK_DATA = {
  "lastUpdate": 1756243927974,
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
            "email": "kai@kaimast.com",
            "name": "Kai Mast",
            "username": "kaimast"
          },
          "distinct": true,
          "id": "7f1bede427914d7641b45cb315df7d877754f756",
          "message": "ci: add sync benchmark with 40 validator ledger",
          "timestamp": "2025-08-24T12:30:06-07:00",
          "tree_id": "812fe5de08421f1c319f69ccea65c2fa93df4b9a",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/7f1bede427914d7641b45cb315df7d877754f756"
        },
        "date": 1756066615171,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.28,
            "unit": "blocks/s",
            "extra": "total_wait=848s"
          },
          {
            "name": "cdn-sync",
            "value": 0.92,
            "unit": "blocks/s",
            "extra": "total_wait=269s"
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
            "email": "kai@kaimast.com",
            "name": "Kai Mast",
            "username": "kaimast"
          },
          "distinct": true,
          "id": "97fffa7d12d7747b4481641632e1fd1e6782ec9a",
          "message": "ci: add script to generate sync ledger",
          "timestamp": "2025-08-25T23:43:32-07:00",
          "tree_id": "1733385a3c8539590822d4eb45273a64f3d4cba8",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/97fffa7d12d7747b4481641632e1fd1e6782ec9a"
        },
        "date": 1756193686768,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.24,
            "unit": "blocks/s",
            "extra": "total_wait=1018s, target_height=250, "
          },
          {
            "name": "cdn-sync",
            "value": 0.89,
            "unit": "blocks/s",
            "extra": "total_wait=279s, target_height=250"
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
          "id": "b96a6549dab49c1204f42e7461959fa594301779",
          "message": "ci: add trap handler for nodes quitting early",
          "timestamp": "2025-08-26T12:50:56-07:00",
          "tree_id": "7984d9347de176bbaed2678e042b6f81f3cc5333",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/b96a6549dab49c1204f42e7461959fa594301779"
        },
        "date": 1756240752448,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.31,
            "unit": "blocks/s",
            "extra": "total_wait=782s, target_height=250, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.88,
            "unit": "blocks/s",
            "extra": "total_wait=283s, target_height=250"
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
          "id": "f8fb275e016b6fe32e4e32ba1485431ee7d32545",
          "message": "ci: add trap handler for nodes quitting early",
          "timestamp": "2025-08-26T13:40:12-07:00",
          "tree_id": "7df9d5609f2363c405ed99592839f056d7ea857a",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/f8fb275e016b6fe32e4e32ba1485431ee7d32545"
        },
        "date": 1756243927274,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.2,
            "unit": "blocks/s",
            "extra": "total_wait=1227s, target_height=250, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.85,
            "unit": "blocks/s",
            "extra": "total_wait=291s, target_height=250"
          }
        ]
      }
    ]
  }
}

window.BENCHMARK_DATA = {
  "lastUpdate": 1756502558056,
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
          "id": "331f624c23086912877b5cc9fc6c8662713b72d9",
          "message": "ci: add trap handler for nodes quitting early",
          "timestamp": "2025-08-26T16:18:08-07:00",
          "tree_id": "6400900496b8221008acb88ab276330ff2c86147",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/331f624c23086912877b5cc9fc6c8662713b72d9"
        },
        "date": 1756253734159,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.2,
            "unit": "blocks/s",
            "extra": "total_wait=1213s, target_height=250, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.87,
            "unit": "blocks/s",
            "extra": "total_wait=287s, target_height=250"
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
          "id": "89075ad9c7fbc0c1226ed9ead7e45e42cec36b7d",
          "message": "ci: measure sync speed variance",
          "timestamp": "2025-08-26T20:50:00-07:00",
          "tree_id": "f2fecbc3288687cbd98527605c381782c39e64c5",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/89075ad9c7fbc0c1226ed9ead7e45e42cec36b7d"
        },
        "date": 1756269719083,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.21,
            "unit": "blocks/s",
            "extra": "total_wait=1141s, target_height=250, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.131204,
            "unit": "blocks^2/s^2",
            "extra": "samples=1142, mean_bps=0.107089, max_speed=0, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.93,
            "unit": "blocks/s",
            "extra": "total_wait=267s, target_height=250"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "victor.s.nicolaas@protonmail.com",
            "name": "vicsn",
            "username": "vicsn"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7c3e067b6eaef36b9df95e58a00138f4f6bf18ce",
          "message": "Merge pull request #3783 from ProvableHQ/ci/sync-benchmark\n\n[CI] Sync benchmarks",
          "timestamp": "2025-08-27T09:53:37+02:00",
          "tree_id": "3261fcab9d4366231309c6b9e27f570f081b2655",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/7c3e067b6eaef36b9df95e58a00138f4f6bf18ce"
        },
        "date": 1756284501484,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.28,
            "unit": "blocks/s",
            "extra": "total_wait=874s, target_height=250, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.9,
            "unit": "blocks/s",
            "extra": "total_wait=275s, target_height=250"
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
          "id": "f88eceb445a841f722a3281f73a8d0ada74bb9aa",
          "message": "ci: measure sync speed variance",
          "timestamp": "2025-08-27T09:49:48-07:00",
          "tree_id": "985faf5e02d33f9ff6a08e4bcbefc1a0980c060e",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/f88eceb445a841f722a3281f73a8d0ada74bb9aa"
        },
        "date": 1756316289414,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.29,
            "unit": "blocks/s",
            "extra": "total_wait=843s, target_height=250, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.028021,
            "unit": "blocks^2/s^2",
            "extra": "samples=844, mean_bps=0.045023, max_speed=0, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.91,
            "unit": "blocks/s",
            "extra": "total_wait=272s, target_height=250"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "kai@kaimast.com",
            "name": "Kai Mast",
            "username": "kaimast"
          },
          "committer": {
            "email": "kai@kaimast.com",
            "name": "Kai Mast",
            "username": "kaimast"
          },
          "distinct": true,
          "id": "666746efb9fb0e2b952c786d70a1f8cdaad6f44b",
          "message": "ci: enable BFT benchmark",
          "timestamp": "2025-08-27T16:28:54-07:00",
          "tree_id": "a927caa760df1eed8ec3bdc450ca255d74a78dfd",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/666746efb9fb0e2b952c786d70a1f8cdaad6f44b"
        },
        "date": 1756341158437,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.46,
            "unit": "blocks/s",
            "extra": "total_wait=536s, target_height=250, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.451896,
            "unit": "blocks^2/s^2",
            "extra": "samples=537, mean_speed=0.291365, max_speed=2, branch=ci/sync-variance-benchmark, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 1.86,
            "unit": "ops/s",
            "extra": "num_get_ops=1000, base_url=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.23,
            "unit": "blocks/s",
            "extra": "total_wait=1042s, target_height=250, branch=ci/sync-variance-benchmark, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.94,
            "unit": "blocks/s",
            "extra": "total_wait=265s, target_height=250"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "kai@kaimast.com",
            "name": "Kai Mast",
            "username": "kaimast"
          },
          "committer": {
            "email": "kai@kaimast.com",
            "name": "Kai Mast",
            "username": "kaimast"
          },
          "distinct": true,
          "id": "e19fd73571431151bef2269167c8175ab8c07293",
          "message": "ci: do not measure initial handshake time",
          "timestamp": "2025-08-27T23:01:40-07:00",
          "tree_id": "e90a871512c0f16edaf54c8f0d9c9d99393ae462",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/e19fd73571431151bef2269167c8175ab8c07293"
        },
        "date": 1756364522569,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.19,
            "unit": "blocks/s",
            "extra": "total_wait=1292s, target_height=250, connect_time=9s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.183266,
            "unit": "blocks^2/s^2",
            "extra": "samples=1230, mean_speed=0.126881, max_speed=2, branch=ci/sync-variance-benchmark, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.77,
            "unit": "ops/s",
            "extra": "num_get_ops=1000, base_url=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.44,
            "unit": "blocks/s",
            "extra": "total_wait=565s, target_height=250"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "kai@kaimast.com",
            "name": "Kai Mast",
            "username": "kaimast"
          },
          "committer": {
            "email": "kai@kaimast.com",
            "name": "Kai Mast",
            "username": "kaimast"
          },
          "distinct": true,
          "id": "71bf68148cb4fe04010b6385b5359d4a04b269a8",
          "message": "ci: fix typo and show elapsed time as minutes",
          "timestamp": "2025-08-29T12:43:09-07:00",
          "tree_id": "b713e208459b9d4fba13bd35142a6e7d910ac05d",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/71bf68148cb4fe04010b6385b5359d4a04b269a8"
        },
        "date": 1756502557141,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.19,
            "unit": "blocks/s",
            "extra": "total_wait=1288s, target_height=250, connect_time=6s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.281965,
            "unit": "blocks^2/s^2",
            "extra": "samples=1236, mean_speed=0.165723, max_speed=2.900000, branch=ci/sync-variance-benchmark, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.61,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=809, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 185.18,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=54, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.17,
            "unit": "blocks/s",
            "extra": "total_wait=1431s, target_height=250, connect_time=0, branch=ci/sync-variance-benchmark, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.44,
            "unit": "blocks/s",
            "extra": "total_wait=566s, target_height=250"
          }
        ]
      }
    ]
  }
}

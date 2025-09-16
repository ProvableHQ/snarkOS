window.BENCHMARK_DATA = {
  "lastUpdate": 1758032262458,
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
          "id": "e43e67d845259a1aedac2e393c515ad4ea418378",
          "message": "ci: report rest-block-height correctly",
          "timestamp": "2025-08-29T14:34:33-07:00",
          "tree_id": "5d7a9c9d772096370bbe9924558b99a2738e6bdd",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/e43e67d845259a1aedac2e393c515ad4ea418378"
        },
        "date": 1756508056945,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.31,
            "unit": "blocks/s",
            "extra": "total_wait=783s, target_height=250, connect_time=11s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.281402,
            "unit": "blocks^2/s^2",
            "extra": "samples=724, mean_speed=0.177716, max_speed=2.466667, branch=ci/sync-variance-benchmark, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.65,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=768, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 185.18,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=54, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.27,
            "unit": "blocks/s",
            "extra": "total_wait=918s, target_height=250, connect_time=0, branch=ci/sync-variance-benchmark, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.44,
            "unit": "blocks/s",
            "extra": "total_wait=564s, target_height=250"
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
          "id": "7a38b9eb20694174d453dc92a157bf306c97fbc9",
          "message": "fix(sync): only count successful block requests for sync speed",
          "timestamp": "2025-08-29T16:50:13-07:00",
          "tree_id": "ad7c40a449165b5568e2b2b52452564aadc3f0e1",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/7a38b9eb20694174d453dc92a157bf306c97fbc9"
        },
        "date": 1756516536264,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.21,
            "unit": "blocks/s",
            "extra": "total_wait=1167s, target_height=250, connect_time=5s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.179385,
            "unit": "blocks^2/s^2",
            "extra": "samples=1123, mean_speed=0.143425, max_speed=2.066667, branch=ci/sync-variance-benchmark, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.64,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=779, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 181.81,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=55, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.28,
            "unit": "blocks/s",
            "extra": "total_wait=863s, target_height=250, connect_time=0, branch=ci/sync-variance-benchmark, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.43,
            "unit": "blocks/s",
            "extra": "total_wait=572s, target_height=250"
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
          "id": "e0e6273531b9377a5e7fdbd84a02d74e6852df5a",
          "message": "ci: print rest benchmark results correctly",
          "timestamp": "2025-08-29T22:55:00-07:00",
          "tree_id": "1eead065944892e49c8262a9c569136a85b24193",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/e0e6273531b9377a5e7fdbd84a02d74e6852df5a"
        },
        "date": 1756538508856,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.27,
            "unit": "blocks/s",
            "extra": "total_wait=906s, target_height=250, connect_time=9s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.343061,
            "unit": "blocks^2/s^2",
            "extra": "samples=873, mean_speed=0.153341, max_speed=3.533333, branch=ci/sync-variance-benchmark, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.65,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=764, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 185.18,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=54, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.19,
            "unit": "blocks/s",
            "extra": "total_wait=1314s, target_height=250, connect_time=0, branch=ci/sync-variance-benchmark, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.46,
            "unit": "blocks/s",
            "extra": "total_wait=539s, target_height=250"
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
          "id": "9d9099dc0ef1a0cac2abe50def3d8251573bb82d",
          "message": "ci: fix typo",
          "timestamp": "2025-08-30T11:05:37-07:00",
          "tree_id": "4ff83634702c20967d4c8aae9d267980839089a0",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/9d9099dc0ef1a0cac2abe50def3d8251573bb82d"
        },
        "date": 1756581873755,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.31,
            "unit": "blocks/s",
            "extra": "total_wait=791s, target_height=250, connect_time=7s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.249581,
            "unit": "blocks^2/s^2",
            "extra": "samples=767, mean_speed=0.121295, max_speed=3.900000, branch=ci/sync-variance-benchmark, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.7,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=707, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 196.07,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=51, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.23,
            "unit": "blocks/s",
            "extra": "total_wait=1074s, target_height=250, connect_time=0, branch=ci/sync-variance-benchmark, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.46,
            "unit": "blocks/s",
            "extra": "total_wait=534s, target_height=250"
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
          "id": "50c811436a41cd6691368ecd0d6233544116dfe0",
          "message": "Merge pull request #3802 from ProvableHQ/ci/sync-variance-benchmark\n\n[CI] Sync variance benchmark",
          "timestamp": "2025-09-01T09:52:39+02:00",
          "tree_id": "4ff83634702c20967d4c8aae9d267980839089a0",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/50c811436a41cd6691368ecd0d6233544116dfe0"
        },
        "date": 1756718442447,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.31,
            "unit": "blocks/s",
            "extra": "total_wait=805s, target_height=250, connect_time=13s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.40783,
            "unit": "blocks^2/s^2",
            "extra": "samples=774, mean_speed=0.203811, max_speed=3.050000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.63,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=788, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 175.43,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=57, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.29,
            "unit": "blocks/s",
            "extra": "total_wait=835s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.43,
            "unit": "blocks/s",
            "extra": "total_wait=570s, target_height=250"
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
          "id": "6f3cc5c4377f5aa0180417c16ba4b5222ec6e208",
          "message": "Merge pull request #3816 from ProvableHQ/canary-postrelease-merge\n\nCanary postrelease merge",
          "timestamp": "2025-09-01T16:35:29+02:00",
          "tree_id": "9b10edfa3957b4056cf1f73e128dcd686bf90ce0",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/6f3cc5c4377f5aa0180417c16ba4b5222ec6e208"
        },
        "date": 1756743050256,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.24,
            "unit": "blocks/s",
            "extra": "total_wait=1012s, target_height=250, connect_time=4s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.117183,
            "unit": "blocks^2/s^2",
            "extra": "samples=970, mean_speed=0.136804, max_speed=2.500000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.64,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=775, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 172.41,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=58, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.16,
            "unit": "blocks/s",
            "extra": "total_wait=1474s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.43,
            "unit": "blocks/s",
            "extra": "total_wait=569s, target_height=250"
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
          "id": "111ae109b37c8834f8d8dfd2ba53279ae77aef3c",
          "message": "Merge pull request #3815 from ProvableHQ/snarkvm-update\n\nsnarkVM rev update",
          "timestamp": "2025-09-01T18:02:12+02:00",
          "tree_id": "e95b1e62b51421405824786375b390a44cc20e6e",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/111ae109b37c8834f8d8dfd2ba53279ae77aef3c"
        },
        "date": 1756748470288,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.2,
            "unit": "blocks/s",
            "extra": "total_wait=1196s, target_height=250, connect_time=8s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.110942,
            "unit": "blocks^2/s^2",
            "extra": "samples=1150, mean_speed=0.101884, max_speed=3.100000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.65,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=760, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 185.18,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=54, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.24,
            "unit": "blocks/s",
            "extra": "total_wait=1020s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.44,
            "unit": "blocks/s",
            "extra": "total_wait=567s, target_height=250"
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
          "id": "b181bfe9d1b37f47c10b78d68d7eaf07f0dcb06f",
          "message": "Merge pull request #3681 from ProvableHQ/feat/pending-blocks\n\n[Refactor] Use PendingBlocks API of snarkVM",
          "timestamp": "2025-09-01T22:15:26+02:00",
          "tree_id": "015595967fabe6532361044d489d59b4f62cb433",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/b181bfe9d1b37f47c10b78d68d7eaf07f0dcb06f"
        },
        "date": 1756762685599,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.25,
            "unit": "blocks/s",
            "extra": "total_wait=978s, target_height=250, connect_time=9s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.354921,
            "unit": "blocks^2/s^2",
            "extra": "samples=943, mean_speed=0.163538, max_speed=3.166667, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.63,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=784, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 181.81,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=55, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.28,
            "unit": "blocks/s",
            "extra": "total_wait=868s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.44,
            "unit": "blocks/s",
            "extra": "total_wait=568s, target_height=250"
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
          "id": "9a13681eb8ee20c75fe5458b6dced26402752b16",
          "message": "Merge pull request #3819 from ProvableHQ/fix-package-deploy\n\nUse a process with loaded imports for deploymeny cost determination",
          "timestamp": "2025-09-02T09:36:07+02:00",
          "tree_id": "b8549d3c9aac2368a850d1cf1e4c502b2e5a7e78",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/9a13681eb8ee20c75fe5458b6dced26402752b16"
        },
        "date": 1756804351926,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.13,
            "unit": "blocks/s",
            "extra": "total_wait=1805s, target_height=250, connect_time=9s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.078919,
            "unit": "blocks^2/s^2",
            "extra": "samples=1718, mean_speed=0.064823, max_speed=3.266667, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.68,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=726, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 185.18,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=54, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.29,
            "unit": "blocks/s",
            "extra": "total_wait=856s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
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
          "id": "ebb491e79483ed1b08e1bd06827ae930d2d3c324",
          "message": "Merge pull request #3818 from ProvableHQ/copilot/fix-3817\n\nSplit CI workflows into focused groups for better failure isolation",
          "timestamp": "2025-09-03T10:46:16+02:00",
          "tree_id": "55ebc8119f008a269ed2c898945efb0105d0da97",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/ebb491e79483ed1b08e1bd06827ae930d2d3c324"
        },
        "date": 1756894928451,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.18,
            "unit": "blocks/s",
            "extra": "total_wait=1320s, target_height=250, connect_time=10s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.130205,
            "unit": "blocks^2/s^2",
            "extra": "samples=1268, mean_speed=0.103194, max_speed=2.783333, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.65,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=763, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 185.18,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=54, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.18,
            "unit": "blocks/s",
            "extra": "total_wait=1322s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.44,
            "unit": "blocks/s",
            "extra": "total_wait=562s, target_height=250"
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
          "id": "97b0eebf412ded0525e9f9bf961ac942dd3234c4",
          "message": "Merge pull request #3825 from ProvableHQ/copilot/fix-3824\n\nIgnore SYNC_LENIENCY for broadcast",
          "timestamp": "2025-09-05T15:53:24+02:00",
          "tree_id": "4cc271a35922651c6ecc2bc6fbaf6028320223af",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/97b0eebf412ded0525e9f9bf961ac942dd3234c4"
        },
        "date": 1757085872925,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.25,
            "unit": "blocks/s",
            "extra": "total_wait=966s, target_height=250, connect_time=10s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.182989,
            "unit": "blocks^2/s^2",
            "extra": "samples=927, mean_speed=0.182920, max_speed=1.450000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.65,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=768, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 166.66,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=60, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.21,
            "unit": "blocks/s",
            "extra": "total_wait=1171s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.44,
            "unit": "blocks/s",
            "extra": "total_wait=563s, target_height=250"
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
          "id": "31f9ccad45816d57856dbf42845a0bbe203196e3",
          "message": "Merge pull request #3823 from tenequm/fix/update-outdated-dependencies\n\nfix: Update tikv-jemallocator and clap to resolve outdated dependencies",
          "timestamp": "2025-09-05T16:03:25+02:00",
          "tree_id": "ea52f28775f6f8869325fc5394a71ed38ef2fa93",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/31f9ccad45816d57856dbf42845a0bbe203196e3"
        },
        "date": 1757087159896,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.25,
            "unit": "blocks/s",
            "extra": "total_wait=979s, target_height=250, connect_time=7s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.139136,
            "unit": "blocks^2/s^2",
            "extra": "samples=940, mean_speed=0.142092, max_speed=2.416667, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.62,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=805, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 181.81,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=55, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.18,
            "unit": "blocks/s",
            "extra": "total_wait=1378s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.44,
            "unit": "blocks/s",
            "extra": "total_wait=562s, target_height=250"
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
          "id": "8d1649f80272a69b1f3171e718dd823540f8ba57",
          "message": "Merge pull request #3820 from ljedrz/feat/peer_cache\n\n[Feat] Persistent peer cache for clients",
          "timestamp": "2025-09-08T20:47:37+02:00",
          "tree_id": "54fdebd69f62ecc29bceecb331003143fa30295e",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/8d1649f80272a69b1f3171e718dd823540f8ba57"
        },
        "date": 1757362322950,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.32,
            "unit": "blocks/s",
            "extra": "total_wait=772s, target_height=250, connect_time=13s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.300308,
            "unit": "blocks^2/s^2",
            "extra": "samples=744, mean_speed=0.129928, max_speed=4.033333, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.62,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=799, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 169.49,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=59, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.24,
            "unit": "blocks/s",
            "extra": "total_wait=1006s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.43,
            "unit": "blocks/s",
            "extra": "total_wait=569s, target_height=250"
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
          "id": "be99cde9b3d8b1cbaaa4f1b80d9d8504ef144ba8",
          "message": "Merge pull request #3826 from ProvableHQ/feat/better-errors\n\n[Feature] Ensure the process exits and produces a log message on panics",
          "timestamp": "2025-09-09T19:45:59+02:00",
          "tree_id": "9fdde7c33e68f3efe980fd54a4ca777d24570cba",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/be99cde9b3d8b1cbaaa4f1b80d9d8504ef144ba8"
        },
        "date": 1757444423993,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.28,
            "unit": "blocks/s",
            "extra": "total_wait=872s, target_height=250, connect_time=5s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.31307,
            "unit": "blocks^2/s^2",
            "extra": "samples=847, mean_speed=0.180815, max_speed=3.100000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.64,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=779, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 185.18,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=54, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.26,
            "unit": "blocks/s",
            "extra": "total_wait=957s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.44,
            "unit": "blocks/s",
            "extra": "total_wait=567s, target_height=250"
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
          "id": "3b0a5a291644b0e6063a983a593c3de7bf834112",
          "message": "Merge pull request #3836 from ProvableHQ/fix/endpoint-response\n\n[Fix] CLI endpoint response handling",
          "timestamp": "2025-09-09T23:46:02+02:00",
          "tree_id": "870bf0f3c32f14d03fe471548e3cfdee13c8c1ac",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/3b0a5a291644b0e6063a983a593c3de7bf834112"
        },
        "date": 1757459292187,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.17,
            "unit": "blocks/s",
            "extra": "total_wait=1451s, target_height=250, connect_time=12s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.11203,
            "unit": "blocks^2/s^2",
            "extra": "samples=1409, mean_speed=0.124202, max_speed=1.500000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.62,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=797, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 188.67,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=53, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.26,
            "unit": "blocks/s",
            "extra": "total_wait=951s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
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
          "id": "647f6f325420d994104a9bd3c1062984653e7d1f",
          "message": "Merge pull request #3840 from ProvableHQ/postrelease-merge-mainnet\n\nPostrelease merge mainnet",
          "timestamp": "2025-09-11T23:37:33+02:00",
          "tree_id": "23bcbe85006f312eed7efb950006edd98215965b",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/647f6f325420d994104a9bd3c1062984653e7d1f"
        },
        "date": 1757631253327,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.25,
            "unit": "blocks/s",
            "extra": "total_wait=994s, target_height=250, connect_time=3s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.176444,
            "unit": "blocks^2/s^2",
            "extra": "samples=967, mean_speed=0.109755, max_speed=3.483333, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.66,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=750, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 188.67,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=53, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.26,
            "unit": "blocks/s",
            "extra": "total_wait=941s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.43,
            "unit": "blocks/s",
            "extra": "total_wait=569s, target_height=250"
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "de856fd42d6caa022810742329d7fd7c51e8913c",
          "message": "Merge pull request #3847 from ProvableHQ/fix/revert-pending-blocks\n\n[Fix] Revert `PendingBlocks` PR",
          "timestamp": "2025-09-15T15:24:01-05:00",
          "tree_id": "6660d2d0b0ae7d1e50eaa35dc0cd606ead35f7d4",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/de856fd42d6caa022810742329d7fd7c51e8913c"
        },
        "date": 1757972460203,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.25,
            "unit": "blocks/s",
            "extra": "total_wait=976s, target_height=250, connect_time=5s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.253667,
            "unit": "blocks^2/s^2",
            "extra": "samples=949, mean_speed=0.133298, max_speed=3.150000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.65,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=762, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 188.67,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=53, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.28,
            "unit": "blocks/s",
            "extra": "total_wait=863s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.44,
            "unit": "blocks/s",
            "extra": "total_wait=567s, target_height=250"
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
          "id": "30626d0b9d22facc66a02400b9f50855ed9e6d3c",
          "message": "Merge pull request #3693 from ProvableHQ/stale_transactions_solutions\n\nTrack stale transactions and solutions separately",
          "timestamp": "2025-09-16T10:45:00+02:00",
          "tree_id": "4e25d80268d9fc5162acf974f70d075ccc52bed7",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/30626d0b9d22facc66a02400b9f50855ed9e6d3c"
        },
        "date": 1758016953169,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.26,
            "unit": "blocks/s",
            "extra": "total_wait=942s, target_height=250, connect_time=10s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.18191,
            "unit": "blocks^2/s^2",
            "extra": "samples=916, mean_speed=0.123763, max_speed=3.450000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.68,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=725, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 185.18,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=54, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.19,
            "unit": "blocks/s",
            "extra": "total_wait=1264s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.44,
            "unit": "blocks/s",
            "extra": "total_wait=567s, target_height=250"
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
          "id": "2dba0727125b90d5f80f8d080b43f6c323750f87",
          "message": "Merge pull request #3845 from ProvableHQ/remove_magic_peer_cliff\n\nRemove peer cliff based on magic 0.75 threshold",
          "timestamp": "2025-09-16T10:45:45+02:00",
          "tree_id": "398aacb3c6cf1242bf1ce109250c4cc01d6e3673",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/2dba0727125b90d5f80f8d080b43f6c323750f87"
        },
        "date": 1758017106082,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.2,
            "unit": "blocks/s",
            "extra": "total_wait=1224s, target_height=250, connect_time=24s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.10358,
            "unit": "blocks^2/s^2",
            "extra": "samples=1189, mean_speed=0.134413, max_speed=1.766667, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.64,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=776, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 188.67,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=53, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.27,
            "unit": "blocks/s",
            "extra": "total_wait=897s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.44,
            "unit": "blocks/s",
            "extra": "total_wait=564s, target_height=250"
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
          "id": "1d90fdc55cdf2f3a28738c199b98f1a61e0cea41",
          "message": "Merge pull request #3848 from ljedrz/feat/track_peer_height_for_cache\n\n[Feat] Track peer height for cache purposes",
          "timestamp": "2025-09-16T15:02:48+02:00",
          "tree_id": "00028dcfc7bcc9c74e08ce0194ba2475aae92eaf",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/1d90fdc55cdf2f3a28738c199b98f1a61e0cea41"
        },
        "date": 1758032262015,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.23,
            "unit": "blocks/s",
            "extra": "total_wait=1085s, target_height=250, connect_time=6s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.244794,
            "unit": "blocks^2/s^2",
            "extra": "samples=1054, mean_speed=0.206926, max_speed=1.916667, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.64,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=773, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 185.18,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=54, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.27,
            "unit": "blocks/s",
            "extra": "total_wait=908s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.44,
            "unit": "blocks/s",
            "extra": "total_wait=563s, target_height=250"
          }
        ]
      }
    ]
  }
}
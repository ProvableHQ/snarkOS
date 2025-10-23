window.BENCHMARK_DATA = {
  "lastUpdate": 1761199762634,
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
          "id": "50397db6fa848ba7e5a572a1a59bc6d4b17bd3c2",
          "message": "Merge pull request #3861 from ProvableHQ/doc/routes\n\n[Docs] Use proper doc comments for REST functions",
          "timestamp": "2025-09-18T08:58:31+02:00",
          "tree_id": "c625eed5e276f451a7ecc4211478061b2f80bf6e",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/50397db6fa848ba7e5a572a1a59bc6d4b17bd3c2"
        },
        "date": 1758182969740,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.3,
            "unit": "blocks/s",
            "extra": "total_wait=820s, target_height=250, connect_time=6s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.249921,
            "unit": "blocks^2/s^2",
            "extra": "samples=794, mean_speed=0.124223, max_speed=3.883333, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.65,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=762, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 185.18,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=54, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.25,
            "unit": "blocks/s",
            "extra": "total_wait=970s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.44,
            "unit": "blocks/s",
            "extra": "total_wait=557s, target_height=250"
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
          "id": "ad707be794afa95a94eee80b75d0ea738ec65eaa",
          "message": "Merge pull request #3841 from ProvableHQ/feat/account-import\n\n[Feat] Implement `snarkos account import`",
          "timestamp": "2025-09-18T09:20:29+02:00",
          "tree_id": "2c879738d3de5c2cc840cbefca492a1e54f3e94b",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/ad707be794afa95a94eee80b75d0ea738ec65eaa"
        },
        "date": 1758185213637,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.26,
            "unit": "blocks/s",
            "extra": "total_wait=941s, target_height=250, connect_time=8s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.186823,
            "unit": "blocks^2/s^2",
            "extra": "samples=915, mean_speed=0.112495, max_speed=3.650000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.66,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=747, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 188.67,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=53, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.15,
            "unit": "blocks/s",
            "extra": "total_wait=1593s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
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
          "id": "1d1c1355b2af79a616aafd956487d87d7f01d6bc",
          "message": "Merge pull request #3862 from ljedrz/cleanup/remove_cuda_from_run-prover\n\n[Cleanup] Remove CUDA from run-prover.sh",
          "timestamp": "2025-09-18T09:55:33+02:00",
          "tree_id": "cf1df19803f07675f14fa60989aafd88ad87c301",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/1d1c1355b2af79a616aafd956487d87d7f01d6bc"
        },
        "date": 1758187439850,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.17,
            "unit": "blocks/s",
            "extra": "total_wait=1408s, target_height=250, connect_time=10s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.119061,
            "unit": "blocks^2/s^2",
            "extra": "samples=1366, mean_speed=0.104563, max_speed=3.050000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.67,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=746, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 178.57,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=56, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.25,
            "unit": "blocks/s",
            "extra": "total_wait=963s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
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
          "id": "2c2e843f58fd974932c5aad12a356c338889f230",
          "message": "Merge pull request #3863 from ljedrz/tweak/bootstrap_peers_cant_be_trusted_peers\n\n[Tweak] Disallow marking bootstrap peers as trusted",
          "timestamp": "2025-09-18T14:16:15+02:00",
          "tree_id": "54e8f8ace67dc4742473497fd1e4281766fd0df1",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/2c2e843f58fd974932c5aad12a356c338889f230"
        },
        "date": 1758202897977,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.25,
            "unit": "blocks/s",
            "extra": "total_wait=964s, target_height=250, connect_time=13s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.307969,
            "unit": "blocks^2/s^2",
            "extra": "samples=937, mean_speed=0.124369, max_speed=4.000000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.66,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=750, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 185.18,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=54, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.16,
            "unit": "blocks/s",
            "extra": "total_wait=1533s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
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
          "id": "7fe273b4119140a58fce999cc992c08bbe8e26f6",
          "message": "Merge pull request #3865 from ljedrz/logs/improve_broken_handshake_log\n\n[Logs] Improve the broken handshake log message",
          "timestamp": "2025-09-18T14:40:10+02:00",
          "tree_id": "fd78612eda1c8cf6cc16aa004246ac5efcf9edbe",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/7fe273b4119140a58fce999cc992c08bbe8e26f6"
        },
        "date": 1758204136190,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.23,
            "unit": "blocks/s",
            "extra": "total_wait=1057s, target_height=250, connect_time=6s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.191816,
            "unit": "blocks^2/s^2",
            "extra": "samples=998, mean_speed=0.139679, max_speed=2.100000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.66,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=749, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 188.67,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=53, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.19,
            "unit": "blocks/s",
            "extra": "total_wait=1279s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
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
          "id": "725734f6a1df02adaf8bb03dd38983776828cdb5",
          "message": "Merge pull request #3870 from ProvableHQ/ci/transaction-unconfirmed-endpoint\n\n[CI] Check transaction/unconfirmed endpoint",
          "timestamp": "2025-09-19T16:26:18+02:00",
          "tree_id": "6534bf645076a485038a85d5e755050b14c288ca",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/725734f6a1df02adaf8bb03dd38983776828cdb5"
        },
        "date": 1758296202360,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.28,
            "unit": "blocks/s",
            "extra": "total_wait=878s, target_height=250, connect_time=6s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.172923,
            "unit": "blocks^2/s^2",
            "extra": "samples=854, mean_speed=0.127674, max_speed=3.316667, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.64,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=780, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 188.67,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=53, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.29,
            "unit": "blocks/s",
            "extra": "total_wait=845s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
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
          "id": "b800b12c95fe4a476c6a190baccb2aed6b3a5f27",
          "message": "Merge pull request #3877 from ljedrz/feat/host_resolution_for_validators\n\n[Feat] Allow trusted validator addresses to include hostnames",
          "timestamp": "2025-09-24T09:14:35+02:00",
          "tree_id": "295febac73cb1794ccbb8227231e935d8e10147a",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/b800b12c95fe4a476c6a190baccb2aed6b3a5f27"
        },
        "date": 1758702400368,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.27,
            "unit": "blocks/s",
            "extra": "total_wait=900s, target_height=250, connect_time=7s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.191773,
            "unit": "blocks^2/s^2",
            "extra": "samples=875, mean_speed=0.145105, max_speed=3.300000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
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
            "value": 0.26,
            "unit": "blocks/s",
            "extra": "total_wait=933s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
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
          "id": "f9d4f77f7518cc9e6f534c7cf59cc50396b374d2",
          "message": "Merge pull request #3883 from ProvableHQ/improve_solution_broadcast_staging\n\nEnsure solutions can be broadcast even if not synced",
          "timestamp": "2025-09-24T11:04:25+02:00",
          "tree_id": "d7876d87129a12e8a380ba7418259577025ce5e8",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/f9d4f77f7518cc9e6f534c7cf59cc50396b374d2"
        },
        "date": 1758709204177,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.23,
            "unit": "blocks/s",
            "extra": "total_wait=1058s, target_height=250, connect_time=6s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.238708,
            "unit": "blocks^2/s^2",
            "extra": "samples=1028, mean_speed=0.175324, max_speed=2.250000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.66,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=748, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 188.67,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=53, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.25,
            "unit": "blocks/s",
            "extra": "total_wait=992s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
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
          "id": "b7401cef1c720908670474c4c22a6bc437a51dcf",
          "message": "Merge pull request #3879 from ljedrz/refactor/merge_peers\n\n[Refactor] Align peer handling between Router and Gateway",
          "timestamp": "2025-09-25T11:57:39+02:00",
          "tree_id": "1ccf08248b068cb3e6a7c82b7b801da5d0b1bcbe",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/b7401cef1c720908670474c4c22a6bc437a51dcf"
        },
        "date": 1758798599048,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.25,
            "unit": "blocks/s",
            "extra": "total_wait=964s, target_height=250, connect_time=10s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.243393,
            "unit": "blocks^2/s^2",
            "extra": "samples=936, mean_speed=0.167806, max_speed=3.000000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.67,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=743, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 188.67,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=53, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.29,
            "unit": "blocks/s",
            "extra": "total_wait=839s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
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
          "id": "83805f68a59640c0861218be289d2a77012165b9",
          "message": "Merge pull request #3887 from ljedrz/feat/validator_peer_cache\n\n[Feat] Introduce a persistent validator cache",
          "timestamp": "2025-09-26T12:15:26+02:00",
          "tree_id": "0c343bc43333655993c64e5f4bf9793f94f41fbf",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/83805f68a59640c0861218be289d2a77012165b9"
        },
        "date": 1758886418769,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "p2p-sync",
            "value": 0.25,
            "unit": "blocks/s",
            "extra": "total_wait=982s, target_height=250, connect_time=7s, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.202773,
            "unit": "blocks^2/s^2",
            "extra": "samples=954, mean_speed=0.178634, max_speed=2.300000, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-get-block",
            "value": 0.65,
            "unit": "ops/s",
            "extra": "num_ops=500, total_wait=762, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 185.18,
            "unit": "ops/s",
            "extra": "num_ops=10000, total_wait=54, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 0.2,
            "unit": "blocks/s",
            "extra": "total_wait=1214s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=44b334d398, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 0.43,
            "unit": "blocks/s",
            "extra": "total_wait=573s, target_height=250"
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
          "id": "70966a22bedee1470135d46c0c6d44e181a485f6",
          "message": "Merge pull request #3898 from ProvableHQ/ci/fix-benchmarks\n\n[CI] Update sync snapshot for benchmark",
          "timestamp": "2025-09-30T11:55:08+02:00",
          "tree_id": "4a90012ab1a2531010ed45deafea47fc31441d19",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/70966a22bedee1470135d46c0c6d44e181a485f6"
        },
        "date": 1759228054535,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.5682262681233685,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=186.89942002296448, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7959.342360035839,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.051081657409668, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.38,
            "unit": "blocks/s",
            "extra": "total_wait=181s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.333612,
            "unit": "blocks^2/s^2",
            "extra": "samples=176, mean_speed=0.860322, max_speed=3.233333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.16,
            "unit": "blocks/s",
            "extra": "total_wait=214s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=203s, target_height=250"
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
          "id": "c8bd4304e481bebe4fecc7cb684c7056ea1a6162",
          "message": "Merge pull request #3894 from ljedrz/tweak/improve_peer_cache_connection_setup\n\n[Logs] Reduce the log level when failing to connect to non-trusted/bootstrap peers",
          "timestamp": "2025-09-30T14:02:22+02:00",
          "tree_id": "b414b2fb629fd4d231b1b07c8dae20ec8327d895",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/c8bd4304e481bebe4fecc7cb684c7056ea1a6162"
        },
        "date": 1759235518720,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.7728006145907176,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=173.11017513275146, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 8022.948867498141,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=9.97139596939087, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.3,
            "unit": "blocks/s",
            "extra": "total_wait=192s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.187566,
            "unit": "blocks^2/s^2",
            "extra": "samples=187, mean_speed=0.882175, max_speed=2.866667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.17,
            "unit": "blocks/s",
            "extra": "total_wait=213s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250"
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
          "id": "16d18aeec1e29112d01a124e9379434f2449d18a",
          "message": "Merge pull request #3901 from ljedrz/tests/banning\n\n[Tests] Add IP-banning test cases",
          "timestamp": "2025-09-30T16:03:14+02:00",
          "tree_id": "097ffb6d18f6f8d200c2f61f71fb2108e7a347b1",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/16d18aeec1e29112d01a124e9379434f2449d18a"
        },
        "date": 1759242769430,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.573361235297701,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=186.52647495269775, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7842.151585338053,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.201282024383545, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.35,
            "unit": "blocks/s",
            "extra": "total_wait=184s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.331295,
            "unit": "blocks^2/s^2",
            "extra": "samples=179, mean_speed=0.833892, max_speed=3.250000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=209s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.24,
            "unit": "blocks/s",
            "extra": "total_wait=201s, target_height=250"
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
          "id": "564b132591a61aacfa9213fd42fa31e587a498f3",
          "message": "Merge pull request #3891 from ProvableHQ/backup_utils\n\nCreate and document native backup utility",
          "timestamp": "2025-10-01T11:58:19+02:00",
          "tree_id": "7c173d6fa9603d374c9a3e55f8b38c2e289df1ee",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/564b132591a61aacfa9213fd42fa31e587a498f3"
        },
        "date": 1759314469707,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.626866696146317,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=182.727201461792, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7945.29979907795,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.068845987319946, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.35,
            "unit": "blocks/s",
            "extra": "total_wait=185s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.201019,
            "unit": "blocks^2/s^2",
            "extra": "samples=181, mean_speed=0.793831, max_speed=3.250000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.17,
            "unit": "blocks/s",
            "extra": "total_wait=212s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=203s, target_height=250"
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
          "id": "2b086093d48105c154fa331a1e5ffe4ccc67c15b",
          "message": "Merge pull request #3900 from ljedrz/fix/dedup_validator_heartbeat_conns\n\n[Fix] Dedup validator heartbeat conns & don't downgrade peers on connection duplicates",
          "timestamp": "2025-10-01T12:16:18-05:00",
          "tree_id": "ed3f50f09af9941d63510a2dc6ab9d80ef82c973",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/2b086093d48105c154fa331a1e5ffe4ccc67c15b"
        },
        "date": 1759340753699,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.5853778304016237,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=185.659517288208, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7893.360970013213,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.135099649429321, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.3,
            "unit": "blocks/s",
            "extra": "total_wait=192s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.862365,
            "unit": "blocks^2/s^2",
            "extra": "samples=187, mean_speed=0.759447, max_speed=2.750000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=203s, target_height=250"
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
          "id": "00b65eb9af39a8444f4367b85bb520c986c0842c",
          "message": "Merge pull request #3908 from ProvableHQ/postrelease-merge-mainnet\n\nPostrelease merge mainnet",
          "timestamp": "2025-10-03T12:42:52+02:00",
          "tree_id": "0e25bbd5248e932745f8868302534b89aa0b6d2a",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/00b65eb9af39a8444f4367b85bb520c986c0842c"
        },
        "date": 1759490111684,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6849022908069338,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=178.7774555683136, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7907.588408653082,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.116864442825317, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.32,
            "unit": "blocks/s",
            "extra": "total_wait=188s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.117211,
            "unit": "blocks^2/s^2",
            "extra": "samples=183, mean_speed=0.817304, max_speed=2.916667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=203s, target_height=250"
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
          "id": "ace50270fbdb7501317e21d357c7c101420ffc54",
          "message": "Merge pull request #3912 from ljedrz/logs/fix_owner_printing\n\n[Logs] Correctly print the PeerPoolHandler owner",
          "timestamp": "2025-10-03T16:48:51+02:00",
          "tree_id": "2a62d5b5739c3851ee1f487c555040c28ad96849",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/ace50270fbdb7501317e21d357c7c101420ffc54"
        },
        "date": 1759504715905,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.7017679080644843,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=177.66144847869873, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 8011.243195998412,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=9.985965728759766, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.32,
            "unit": "blocks/s",
            "extra": "total_wait=188s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.302848,
            "unit": "blocks^2/s^2",
            "extra": "samples=183, mean_speed=0.889982, max_speed=3.250000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.21,
            "unit": "blocks/s",
            "extra": "total_wait=206s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250"
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
          "id": "9c6e42dfea941a4044857a592f6c8ad6ff3e9c5c",
          "message": "Merge pull request #3867 from ProvableHQ/improve_propagation\n\nPropagate transactions even if global state root not found",
          "timestamp": "2025-10-04T08:34:49+02:00",
          "tree_id": "efd51a5e80ea52cfe5e65fb4cccde9698c90b8ff",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/9c6e42dfea941a4044857a592f6c8ad6ff3e9c5c"
        },
        "date": 1759561573044,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6869257186283995,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=178.6428246498108, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7929.325977214104,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.08912992477417, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.36,
            "unit": "blocks/s",
            "extra": "total_wait=183s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.302741,
            "unit": "blocks^2/s^2",
            "extra": "samples=178, mean_speed=0.829307, max_speed=3.416667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=203s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250"
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
          "id": "f3329a17c2abfd568dbe40ce1457a830eabb26d4",
          "message": "Merge pull request #3814 from ProvableHQ/copilot/fix-3813\n\n[Feature] Add GitHub Actions workflow to automatically update snarkVM dependency from multiple branches with PR creation",
          "timestamp": "2025-10-04T09:37:27+02:00",
          "tree_id": "4478dc9adb89d264b2556f4797130c14c798eb3e",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/f3329a17c2abfd568dbe40ce1457a830eabb26d4"
        },
        "date": 1759565290746,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.606360295041044,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=184.16486811637878, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7637.382111921783,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.474793434143066, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.32,
            "unit": "blocks/s",
            "extra": "total_wait=188s, target_height=250, connect_time=23s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.133114,
            "unit": "blocks^2/s^2",
            "extra": "samples=183, mean_speed=0.833515, max_speed=3.000000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.2,
            "unit": "blocks/s",
            "extra": "total_wait=207s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250"
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
          "id": "c55f4dcb7f9dc54b40177b8d772c3360c79638cf",
          "message": "Merge pull request #3913 from ProvableHQ/update-snarkvm-staging\n\nUpdate snarkVM to latest staging commit",
          "timestamp": "2025-10-06T13:38:39+02:00",
          "tree_id": "c04d9c83154b7c52c665456c3551d38cd3d8f408",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/c55f4dcb7f9dc54b40177b8d772c3360c79638cf"
        },
        "date": 1759752737187,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6152770213770267,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=183.5369622707367, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7257.29652898535,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=11.023388624191284, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.33,
            "unit": "blocks/s",
            "extra": "total_wait=187s, target_height=250, connect_time=20s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.330769,
            "unit": "blocks^2/s^2",
            "extra": "samples=182, mean_speed=0.876832, max_speed=3.166667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=202s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=203s, target_height=250"
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
          "id": "6ec1d9a40db3c3541f4da381bc8509e2cf55be78",
          "message": "Merge pull request #3914 from ProvableHQ/dynamic_dev_bootstrap_peers\n\nEnable dev nodes and tests to set bootstrap peers",
          "timestamp": "2025-10-06T17:00:21+02:00",
          "tree_id": "de54b43e4460b710fbfc012dd05034e632e02ffd",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/6ec1d9a40db3c3541f4da381bc8509e2cf55be78"
        },
        "date": 1759764607160,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.5533599984150714,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=187.98759293556213, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7945.990880770277,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.067970275878906, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.35,
            "unit": "blocks/s",
            "extra": "total_wait=185s, target_height=250, connect_time=23s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.47146,
            "unit": "blocks^2/s^2",
            "extra": "samples=180, mean_speed=0.912315, max_speed=3.100000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.13,
            "unit": "blocks/s",
            "extra": "total_wait=220s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=203s, target_height=250"
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
          "id": "186c0b666940a48f488a3c10f7cb5c06348c16b7",
          "message": "Merge pull request #3905 from ljedrz/feat/limit_peer_pool_size\n\n[Feat] Limit peer pool size, remove banned peers from the pool",
          "timestamp": "2025-10-07T09:56:11+02:00",
          "tree_id": "7a30de66801409ea27a5910674f5389e07614923",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/186c0b666940a48f488a3c10f7cb5c06348c16b7"
        },
        "date": 1759825570130,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.5698642565352716,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=186.78029346466064, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7691.368346128849,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.401270151138306, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.38,
            "unit": "blocks/s",
            "extra": "total_wait=181s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.279807,
            "unit": "blocks^2/s^2",
            "extra": "samples=176, mean_speed=0.835322, max_speed=3.233333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250"
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
          "id": "afef56610e28be9db36a3215da5e5ebe1e7000cd",
          "message": "Merge pull request #3860 from ljedrz/feat/harden_peer_response_picks\n\n[Feat] Filter out lower-height peers from PeerResponse messages",
          "timestamp": "2025-10-07T11:24:54+02:00",
          "tree_id": "7b51dac8e845428e55adf738a307d5120baa1c62",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/afef56610e28be9db36a3215da5e5ebe1e7000cd"
        },
        "date": 1759830918177,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.603295750625943,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=184.38166308403015, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7822.52273320004,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.226879835128784, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.26,
            "unit": "blocks/s",
            "extra": "total_wait=197s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.090262,
            "unit": "blocks^2/s^2",
            "extra": "samples=192, mean_speed=0.849045, max_speed=2.750000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=210s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250"
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
          "id": "1114eb42700899b539e6d174d34fa8a61741a049",
          "message": "Merge pull request #3924 from ProvableHQ/reduce_failed_workflow_spam\n\nReduce superfluous failed workflow spam",
          "timestamp": "2025-10-10T10:02:26+02:00",
          "tree_id": "25a7c825c06c9ae21fa8359777f7525515032295",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/1114eb42700899b539e6d174d34fa8a61741a049"
        },
        "date": 1760085323234,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.5253425703121906,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=190.07322239875793, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7988.66354812659,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.014190673828125, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.37,
            "unit": "blocks/s",
            "extra": "total_wait=182s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.243756,
            "unit": "blocks^2/s^2",
            "extra": "samples=177, mean_speed=0.834087, max_speed=3.083333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.2,
            "unit": "blocks/s",
            "extra": "total_wait=207s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250"
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
          "id": "17a0bedcde02e578f5467bc5f336d730824c6be3",
          "message": "Merge pull request #3927 from ljedrz/fix/correct_on_connect_addr\n\n[Fix] Correctly resolve the peer address used in on_connect",
          "timestamp": "2025-10-12T16:08:41+02:00",
          "tree_id": "ed71fe23e7ba564f06f99ee034c118ec52fc6e32",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/17a0bedcde02e578f5467bc5f336d730824c6be3"
        },
        "date": 1760280252026,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6240231081645184,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=182.92521834373474, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7955.292938854448,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.056197881698608, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.32,
            "unit": "blocks/s",
            "extra": "total_wait=188s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.258495,
            "unit": "blocks^2/s^2",
            "extra": "samples=183, mean_speed=0.858015, max_speed=3.083333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.16,
            "unit": "blocks/s",
            "extra": "total_wait=215s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=202s, target_height=250"
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
          "id": "acd1d9f54e99c4c27eaaebeb93ee9f715f021d31",
          "message": "Merge pull request #3810 from ProvableHQ/feat/always-advance\n\n[Perf] Block Synchronization Pipeline",
          "timestamp": "2025-10-12T18:34:09+02:00",
          "tree_id": "bcdbe9e4a25f866a21651c9687c722600e54eaae",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/acd1d9f54e99c4c27eaaebeb93ee9f715f021d31"
        },
        "date": 1760288807695,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.5280410367236694,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=189.87033557891846, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7826.619723103012,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.221526384353638, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.38,
            "unit": "blocks/s",
            "extra": "total_wait=181s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.261835,
            "unit": "blocks^2/s^2",
            "extra": "samples=176, mean_speed=0.823390, max_speed=3.016667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=210s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=202s, target_height=250"
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
          "id": "96c7cd16a53c8aaabfb471b44e2992e351abdebd",
          "message": "Merge pull request #3929 from ProvableHQ/devnet_path\n\nAdd option to devnet.sh to run from local path",
          "timestamp": "2025-10-13T12:37:40+02:00",
          "tree_id": "35728ba66140c43f050947e037e9a027ac393543",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/96c7cd16a53c8aaabfb471b44e2992e351abdebd"
        },
        "date": 1760353658306,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.5903384604358353,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=185.3039698600769, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7943.513854309773,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.071109771728516, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.4,
            "unit": "blocks/s",
            "extra": "total_wait=178s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.46043,
            "unit": "blocks^2/s^2",
            "extra": "samples=173, mean_speed=0.842775, max_speed=3.583333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.16,
            "unit": "blocks/s",
            "extra": "total_wait=215s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=202s, target_height=250"
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
          "id": "d267d3eb61ddac2a51db55b7b8902b2733d1e723",
          "message": "Merge pull request #3932 from ProvableHQ/update_rev\n\nUpdate snarkVM rev",
          "timestamp": "2025-10-13T14:37:57+02:00",
          "tree_id": "bfb5359730f91af7ce52827c1c4c14b5350c0ea5",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/d267d3eb61ddac2a51db55b7b8902b2733d1e723"
        },
        "date": 1760360928232,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6437017414578627,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=181.56359791755676, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7990.361776986526,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.012062311172485, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.41,
            "unit": "blocks/s",
            "extra": "total_wait=177s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.320754,
            "unit": "blocks^2/s^2",
            "extra": "samples=173, mean_speed=0.804624, max_speed=3.250000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250"
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
          "id": "403230d3c2ced5abc31b1dc5adea69bae66f41f6",
          "message": "Merge pull request #3938 from ljedrz/chore/update_deps\n\n[Chore] Update tokio",
          "timestamp": "2025-10-15T14:25:41+02:00",
          "tree_id": "f38e1ef65c1274a16f23166ddfbd20c05d7fa678",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/403230d3c2ced5abc31b1dc5adea69bae66f41f6"
        },
        "date": 1760532964604,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.584064561239665,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=185.75387287139893, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7990.643775309343,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.011708974838257, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.32,
            "unit": "blocks/s",
            "extra": "total_wait=188s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.280207,
            "unit": "blocks^2/s^2",
            "extra": "samples=183, mean_speed=0.884882, max_speed=2.916667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.31,
            "unit": "blocks/s",
            "extra": "total_wait=190s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.24,
            "unit": "blocks/s",
            "extra": "total_wait=201s, target_height=250"
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
          "id": "7abd639ffe2b3a8a90755a066d09130ab651b573",
          "message": "Merge pull request #3939 from ljedrz/feat/tokio-console\n\n[Feat] Introduce a tokio_console feature",
          "timestamp": "2025-10-15T16:04:35+02:00",
          "tree_id": "2d3a2892e99d50f3ebdcb9f982a45ba5e2d114a2",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/7abd639ffe2b3a8a90755a066d09130ab651b573"
        },
        "date": 1760538869344,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.5951185438585513,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=184.9626488685608, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 8140.536348031227,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=9.827362298965454, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.39,
            "unit": "blocks/s",
            "extra": "total_wait=179s, target_height=250, connect_time=19s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.194069,
            "unit": "blocks^2/s^2",
            "extra": "samples=175, mean_speed=0.788857, max_speed=3.416667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=202s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250"
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
          "id": "616daf827929eb3538d41fa24f5a96cafbfd19f5",
          "message": "Merge pull request #3940 from ljedrz/tweak/box_per_connection_tasks\n\n[Tweak] Place per-connection tasks in the heap",
          "timestamp": "2025-10-15T16:46:46+02:00",
          "tree_id": "0681c507899cf1ef415f0df9d500a7d596becde8",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/616daf827929eb3538d41fa24f5a96cafbfd19f5"
        },
        "date": 1760541413455,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.583850781430062,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=185.7692415714264, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7740.0771880182265,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.335814237594604, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.35,
            "unit": "blocks/s",
            "extra": "total_wait=184s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.167487,
            "unit": "blocks^2/s^2",
            "extra": "samples=179, mean_speed=0.840410, max_speed=3.000000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.21,
            "unit": "blocks/s",
            "extra": "total_wait=205s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250"
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
          "id": "1c80ea53624d846edfd1c98f09adfea4758b46f1",
          "message": "Merge pull request #3943 from ProvableHQ/fix_addr_print\n\nFix addr print",
          "timestamp": "2025-10-16T14:48:25+02:00",
          "tree_id": "ecc3a5bc21fc35ba80cae989ce77ef2f654d0cd2",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/1c80ea53624d846edfd1c98f09adfea4758b46f1"
        },
        "date": 1760620708907,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6024503804246297,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=184.441556930542, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7890.450335881851,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.138838291168213, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.35,
            "unit": "blocks/s",
            "extra": "total_wait=184s, target_height=250, connect_time=23s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.213113,
            "unit": "blocks^2/s^2",
            "extra": "samples=180, mean_speed=0.813981, max_speed=3.233333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.16,
            "unit": "blocks/s",
            "extra": "total_wait=215s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=202s, target_height=250"
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
          "id": "03007898175fc35a8ed8e661ac24248220ecf43c",
          "message": "Merge pull request #3941 from ljedrz/tweak/slash_known_peers\n\n[Tweak] Slash KnownPeers together with candidate peers",
          "timestamp": "2025-10-16T15:34:51+02:00",
          "tree_id": "d659b273cd6ca55fdd054a7bee3756bfdd69dfd1",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/03007898175fc35a8ed8e661ac24248220ecf43c"
        },
        "date": 1760623508370,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.5633302706877004,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=187.25640058517456, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7948.413908668797,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.064901113510132, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.31,
            "unit": "blocks/s",
            "extra": "total_wait=190s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.122841,
            "unit": "blocks^2/s^2",
            "extra": "samples=185, mean_speed=0.868468, max_speed=2.666667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.18,
            "unit": "blocks/s",
            "extra": "total_wait=211s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250"
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
          "id": "c8b281f0f2f7f7a39c7db85974ece47abf3e8f4f",
          "message": "Merge pull request #3946 from ProvableHQ/log/no-quorum-error\n\n[Logs] Print error when not connected to a quorum of validators",
          "timestamp": "2025-10-17T12:55:49+02:00",
          "tree_id": "584c861d47d4092db1ae953c6be8146b20b51424",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/c8b281f0f2f7f7a39c7db85974ece47abf3e8f4f"
        },
        "date": 1760700387109,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.7196694055856905,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=176.49203944206238, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7864.743528727684,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.171978235244751, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.36,
            "unit": "blocks/s",
            "extra": "total_wait=183s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.173265,
            "unit": "blocks^2/s^2",
            "extra": "samples=178, mean_speed=0.827528, max_speed=2.916667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.18,
            "unit": "blocks/s",
            "extra": "total_wait=211s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=203s, target_height=250"
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
          "id": "d342b5fcd15b713c192f2cec4d83516c3de07f38",
          "message": "Merge pull request #3947 from ProvableHQ/fix/sync-log-nopeers\n\n[Fix] Create a better log message when not connected to any peers.",
          "timestamp": "2025-10-17T13:05:07+02:00",
          "tree_id": "1d0cd019630e322594d2540f5778ffab540b3f97",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/d342b5fcd15b713c192f2cec4d83516c3de07f38"
        },
        "date": 1760701048191,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6360700204266085,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=182.08924508094788, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7983.5389986957025,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.020618677139282, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.35,
            "unit": "blocks/s",
            "extra": "total_wait=184s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.250699,
            "unit": "blocks^2/s^2",
            "extra": "samples=179, mean_speed=0.864804, max_speed=3.083333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.17,
            "unit": "blocks/s",
            "extra": "total_wait=213s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=203s, target_height=250"
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
          "id": "41654e3d60ab05fc1068b756f6151155571e7cdd",
          "message": "Merge pull request #3948 from ljedrz/fix/correct_peer_downgrading_for_validators\n\n[Fix] Correct downgrade_peer_to_candidate for non-Gateway use",
          "timestamp": "2025-10-17T13:46:55+02:00",
          "tree_id": "0109ff5aa5008e64876f31fd6c39d4634353fff8",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/41654e3d60ab05fc1068b756f6151155571e7cdd"
        },
        "date": 1760703404788,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.620111555678821,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=183.19830656051636, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7842.469593058571,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.200868368148804, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.34,
            "unit": "blocks/s",
            "extra": "total_wait=186s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.190678,
            "unit": "blocks^2/s^2",
            "extra": "samples=182, mean_speed=0.840110, max_speed=2.850000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=209s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250"
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
          "id": "8196a9821ab6ca72f4a33c9fcc56bd2c61e663b5",
          "message": "Merge pull request #3950 from ljedrz/logs/dont_error_on_dupe_addr_as_responder\n\n[Logs] Don't return an error when the Aleo address is already connecte…",
          "timestamp": "2025-10-17T15:32:41+02:00",
          "tree_id": "4955009437597e952e228a82b12ac9727072bd35",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/8196a9821ab6ca72f4a33c9fcc56bd2c61e663b5"
        },
        "date": 1760709794063,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.636881906621977,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=182.03318047523499, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7552.771310750896,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.592138528823853, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.35,
            "unit": "blocks/s",
            "extra": "total_wait=184s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.267599,
            "unit": "blocks^2/s^2",
            "extra": "samples=179, mean_speed=0.865084, max_speed=3.066667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.21,
            "unit": "blocks/s",
            "extra": "total_wait=205s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=203s, target_height=250"
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
          "id": "36c35b7214eccd3ff77a0873eef5a26bba70c9eb",
          "message": "Merge pull request #3952 from ProvableHQ/log/downgrade-tracing-subscriber\n\n[Logs] Downgrade `tracing-subscriber` due to breaking color changes",
          "timestamp": "2025-10-18T10:12:38+02:00",
          "tree_id": "39f7b70ffd0dc5018b25039a7ba4bcacc4e33f70",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/36c35b7214eccd3ff77a0873eef5a26bba70c9eb"
        },
        "date": 1760777098544,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.700167848187693,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=177.76672673225403, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7901.916702332455,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.124125957489014, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.29,
            "unit": "blocks/s",
            "extra": "total_wait=193s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.053035,
            "unit": "blocks^2/s^2",
            "extra": "samples=188, mean_speed=0.814982, max_speed=2.750000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.15,
            "unit": "blocks/s",
            "extra": "total_wait=217s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=203s, target_height=250"
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
          "id": "b6ede3bdceab70b30df3f74fd5efc3c5905453bc",
          "message": "Merge pull request #3942 from ljedrz/feat/tighten_bootstrap_validator_sharing\n\n[Feat] Tighten BootstrapClient validator sharing",
          "timestamp": "2025-10-20T13:25:33+02:00",
          "tree_id": "ea06c4c1a912fcac27cc1ee48c9178da7f77a4b2",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/b6ede3bdceab70b30df3f74fd5efc3c5905453bc"
        },
        "date": 1760961620383,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.61391865328756,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=183.63234043121338, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7904.296163101829,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.121078252792358, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.34,
            "unit": "blocks/s",
            "extra": "total_wait=186s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.311606,
            "unit": "blocks^2/s^2",
            "extra": "samples=181, mean_speed=0.876980, max_speed=2.983333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=203s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.21,
            "unit": "blocks/s",
            "extra": "total_wait=205s, target_height=250"
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
          "id": "9f3e1345d9ba912b3f688906126814da32498ec0",
          "message": "Merge pull request #3954 from ProvableHQ/update_pr_template\n\nupdate PR template",
          "timestamp": "2025-10-20T13:51:14+02:00",
          "tree_id": "dbf79abb61e012dc6b34a2179334dd7d36852a0b",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/9f3e1345d9ba912b3f688906126814da32498ec0"
        },
        "date": 1760963008252,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.66512619685443,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=180.10404181480408, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7785.0265246410645,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.276137113571167, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.35,
            "unit": "blocks/s",
            "extra": "total_wait=184s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.291408,
            "unit": "blocks^2/s^2",
            "extra": "samples=179, mean_speed=0.907542, max_speed=2.916667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=210s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250"
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
          "id": "0c7c40d6d1ffdaf2202ad898100a969cad21e901",
          "message": "Merge pull request #3928 from ProvableHQ/fix/cli-scan-minimal\n\n[Fix] Ensure `snarkos developer scan` works on devnets",
          "timestamp": "2025-10-21T11:46:54+02:00",
          "tree_id": "61fa5fe7b8bb5694cddfded24f1917f6980596fa",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/0c7c40d6d1ffdaf2202ad898100a969cad21e901"
        },
        "date": 1761041796623,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.562310938393127,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=187.33089447021484, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7575.060685604827,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.56097149848938, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.34,
            "unit": "blocks/s",
            "extra": "total_wait=186s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.290796,
            "unit": "blocks^2/s^2",
            "extra": "samples=182, mean_speed=0.888919, max_speed=2.916667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=202s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250"
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
          "id": "317467634f8b674c64262ce62c4eaf647f6ee071",
          "message": "Merge pull request #3955 from ljedrz/tweak/reject_duplicate_aleo_addrs_in_router_mode\n\n[Tweak] Validators shouldn't accept duplicate validator connections in router mode",
          "timestamp": "2025-10-21T21:08:38+02:00",
          "tree_id": "cbad4fc59abbe0d3444af18c9905194cbf3b639d",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/317467634f8b674c64262ce62c4eaf647f6ee071"
        },
        "date": 1761075543645,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.663860952027491,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=180.18958520889282, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7923.677880499894,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.09632158279419, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.4,
            "unit": "blocks/s",
            "extra": "total_wait=178s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.376526,
            "unit": "blocks^2/s^2",
            "extra": "samples=173, mean_speed=0.855106, max_speed=3.333333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=203s, target_height=250"
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
          "id": "33ff743430111b6fde79e6038885628a33b011c7",
          "message": "Merge pull request #3953 from ProvableHQ/misc/remove-dev-storage-path\n\n[Fix] Remove `--storage-path` options for `snarkos developer`",
          "timestamp": "2025-10-21T16:18:16-05:00",
          "tree_id": "f263bd786a5916e537c87a623c914aced21b9a9b",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/33ff743430111b6fde79e6038885628a33b011c7"
        },
        "date": 1761083289689,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.646608843460892,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=181.36416387557983, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7976.739050073279,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.029160976409912, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.37,
            "unit": "blocks/s",
            "extra": "total_wait=182s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.362149,
            "unit": "blocks^2/s^2",
            "extra": "samples=177, mean_speed=0.867232, max_speed=3.216667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.17,
            "unit": "blocks/s",
            "extra": "total_wait=212s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.24,
            "unit": "blocks/s",
            "extra": "total_wait=201s, target_height=250"
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
          "id": "4e91590dd15ab6b0aabdac64827ca4335f64eabf",
          "message": "Merge pull request #3957 from ljedrz/refactor/isolate_new_crate\n\n[Refactor] Isolate a new crate",
          "timestamp": "2025-10-22T16:42:51+02:00",
          "tree_id": "d874bd5de4ab98f4a56c1662dff3a60b29b4d8cf",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/4e91590dd15ab6b0aabdac64827ca4335f64eabf"
        },
        "date": 1761145943503,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.5700214076845884,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=186.76887226104736, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 8102.202827152833,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=9.873857975006104, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.37,
            "unit": "blocks/s",
            "extra": "total_wait=182s, target_height=250, connect_time=23s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.367966,
            "unit": "blocks^2/s^2",
            "extra": "samples=178, mean_speed=0.889700, max_speed=3.300000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.34,
            "unit": "blocks/s",
            "extra": "total_wait=186s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=202s, target_height=250"
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
          "id": "476923712e3b4ace412720583ecf9640eca5262a",
          "message": "Merge pull request #3951 from ProvableHQ/log/sync-verbosity\n\nReduce excessive logging in block sync",
          "timestamp": "2025-10-23T00:39:52-05:00",
          "tree_id": "bb34f91122c60204d7bb0540a9cd509926718dc6",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/476923712e3b4ace412720583ecf9640eca5262a"
        },
        "date": 1761199761772,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6396960346626,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=181.83911848068237, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7911.5097450913445,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.111850023269653, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.36,
            "unit": "blocks/s",
            "extra": "total_wait=183s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.375526,
            "unit": "blocks^2/s^2",
            "extra": "samples=178, mean_speed=0.845506, max_speed=3.366667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.24,
            "unit": "blocks/s",
            "extra": "total_wait=201s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.24,
            "unit": "blocks/s",
            "extra": "total_wait=201s, target_height=250"
          }
        ]
      }
    ]
  }
}
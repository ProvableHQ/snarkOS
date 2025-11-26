window.BENCHMARK_DATA = {
  "lastUpdate": 1764179880099,
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
          "id": "a952ec3a152ed20a77f2289f532c878733356d70",
          "message": "Merge pull request #3958 from ProvableHQ/metrics/block-lag\n\n[Metrics] Add BLOCK_LAG histogram",
          "timestamp": "2025-10-23T11:32:34+02:00",
          "tree_id": "0a0c976e0604ab96983f1902ddfe582abee1bb96",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/a952ec3a152ed20a77f2289f532c878733356d70"
        },
        "date": 1761213746301,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.765326950147984,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=173.57802844047546, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 8053.986795750582,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=9.932968854904175, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.34,
            "unit": "blocks/s",
            "extra": "total_wait=186s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.228104,
            "unit": "blocks^2/s^2",
            "extra": "samples=181, mean_speed=0.887385, max_speed=2.916667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.14,
            "unit": "blocks/s",
            "extra": "total_wait=218s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "b5655780eba992c05c3daf501e474a1ecf8abeba",
          "message": "Merge pull request #3963 from ljedrz/fix/locktick_in_node_network\n\n[Fix] Inherit the locktick feature in snarkos-node-network",
          "timestamp": "2025-10-23T14:34:52+02:00",
          "tree_id": "4f1fb6369da4c2f4e0c429782fcfd14947a7e34b",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/b5655780eba992c05c3daf501e474a1ecf8abeba"
        },
        "date": 1761224709294,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6159179470840046,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=183.49199390411377, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 8002.754396497437,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=9.99655818939209, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.35,
            "unit": "blocks/s",
            "extra": "total_wait=184s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.292009,
            "unit": "blocks^2/s^2",
            "extra": "samples=179, mean_speed=0.827374, max_speed=3.333333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.2,
            "unit": "blocks/s",
            "extra": "total_wait=208s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "9357ebd361014248cbdf4eea32957655471cdfbe",
          "message": "Merge pull request #3956 from ProvableHQ/fix/credits-aleo-version\n\n[Fix] Return 0 for latest credits.aleo edition and 404 for missing programs/editions",
          "timestamp": "2025-10-23T17:06:30-05:00",
          "tree_id": "f6698f83cdd1ad47f95bfe47f9d0d6621d760540",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/9357ebd361014248cbdf4eea32957655471cdfbe"
        },
        "date": 1761259295831,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.4053053640731314,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=199.55886149406433, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 10197.283936089982,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=7.845226287841797, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.2,
            "unit": "blocks/s",
            "extra": "total_wait=208s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.0863,
            "unit": "blocks^2/s^2",
            "extra": "samples=203, mean_speed=0.752956, max_speed=3.083333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.08,
            "unit": "blocks/s",
            "extra": "total_wait=230s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.15,
            "unit": "blocks/s",
            "extra": "total_wait=217s, target_height=250"
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
          "id": "7428412a2abbb32d6c589530311aa6228c3f7f17",
          "message": "Merge pull request #3966 from ProvableHQ/chore/update-snarkvm\n\n[Chore] Update snarkVM rev",
          "timestamp": "2025-10-24T14:46:10+02:00",
          "tree_id": "110a646f15fbf1ff378c22ee80e8a58408d370af",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/7428412a2abbb32d6c589530311aa6228c3f7f17"
        },
        "date": 1761312017127,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.211764682824774,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=217.02127885818481, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 9945.72809890184,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=8.043654441833496, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.17,
            "unit": "blocks/s",
            "extra": "total_wait=213s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 0.929938,
            "unit": "blocks^2/s^2",
            "extra": "samples=208, mean_speed=0.809135, max_speed=2.550000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.07,
            "unit": "blocks/s",
            "extra": "total_wait=232s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.15,
            "unit": "blocks/s",
            "extra": "total_wait=216s, target_height=250"
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
          "id": "0ff8f1b7da788bfa0140767c119298946c4de53b",
          "message": "Merge pull request #3936 from ProvableHQ/feat/sync-consensus-version\n\n[Feature] Do not sync when peer consensus version differs",
          "timestamp": "2025-10-24T20:47:01+02:00",
          "tree_id": "d9c868bd267c3c3b51c4a61e0c09f527b3fe290e",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/0ff8f1b7da788bfa0140767c119298946c4de53b"
        },
        "date": 1761333692874,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.345605454123532,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=204.63799619674683, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 10265.518120682378,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=7.793079614639282, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.25,
            "unit": "blocks/s",
            "extra": "total_wait=199s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.422019,
            "unit": "blocks^2/s^2",
            "extra": "samples=194, mean_speed=0.863058, max_speed=3.300000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.04,
            "unit": "blocks/s",
            "extra": "total_wait=239s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.14,
            "unit": "blocks/s",
            "extra": "total_wait=218s, target_height=250"
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
          "id": "fa8eccee9e740b370d3a042bed2fefdebc99b2f4",
          "message": "Merge pull request #3972 from ProvableHQ/fix/stale-transmissions\n\n[Fix] Do not tack transmissions that are added to the mempool",
          "timestamp": "2025-10-26T23:03:38-05:00",
          "tree_id": "173061ae3c00c78078cbd29608616a531f8c1e52",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/fa8eccee9e740b370d3a042bed2fefdebc99b2f4"
        },
        "date": 1761540234317,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.584000583770947,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=185.7584719657898, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 8076.075390784088,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=9.90580153465271, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.35,
            "unit": "blocks/s",
            "extra": "total_wait=184s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.362668,
            "unit": "blocks^2/s^2",
            "extra": "samples=179, mean_speed=0.863128, max_speed=3.166667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "ca0884d292b033316171883698b58d8d925216b0",
          "message": "Merge pull request #3974 from ProvableHQ/improve_consensus_version_tracking\n\nSpecify ConsensusVersion relevant to the task at hand",
          "timestamp": "2025-10-27T10:05:38+01:00",
          "tree_id": "a003ec22a1a3b0a78eb6d92377935d17ecbe3c2f",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/ca0884d292b033316171883698b58d8d925216b0"
        },
        "date": 1761557802448,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.4952750684025617,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=192.3635618686676, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7752.566153231931,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.319163799285889, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.35,
            "unit": "blocks/s",
            "extra": "total_wait=184s, target_height=250, connect_time=23s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.148984,
            "unit": "blocks^2/s^2",
            "extra": "samples=180, mean_speed=0.837407, max_speed=2.933333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "ee8ca4466d19a3166784b42a6ddb8898f4f1c3e0",
          "message": "Merge pull request #3975 from ProvableHQ/fix/block-response-test\n\n[Fix] Correct tests for new BlockResponse",
          "timestamp": "2025-10-27T10:06:57+01:00",
          "tree_id": "f42cf8ca5f1f1134740e98839c7c0b6dfebc36b8",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/ee8ca4466d19a3166784b42a6ddb8898f4f1c3e0"
        },
        "date": 1761557908887,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6809136073849213,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=179.04344201087952, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7985.52846863506,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.01812219619751, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.3,
            "unit": "blocks/s",
            "extra": "total_wait=192s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.195954,
            "unit": "blocks^2/s^2",
            "extra": "samples=187, mean_speed=0.874688, max_speed=2.800000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "d25ad9e258decd3a27497c61d41cb3037c8c88a9",
          "message": "Merge pull request #3971 from ljedrz/feat/provide_repo_sha_over_handshake\n\n[Feat] Provide repo sha over handshake",
          "timestamp": "2025-10-27T11:06:31+01:00",
          "tree_id": "f353db9c38cc132d1200a17969984763a8e9819c",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/d25ad9e258decd3a27497c61d41cb3037c8c88a9"
        },
        "date": 1761561410864,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.5430777736696997,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=188.7476682662964, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7956.233643266937,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.055008888244629, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.34,
            "unit": "blocks/s",
            "extra": "total_wait=186s, target_height=250, connect_time=20s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.160892,
            "unit": "blocks^2/s^2",
            "extra": "samples=181, mean_speed=0.833149, max_speed=2.916667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "1907daa0d242f6158930f2232caad60d082bd869",
          "message": "Merge pull request #3973 from ProvableHQ/cleanup_logs\n\nReduce spurious ERROR logs",
          "timestamp": "2025-10-27T11:27:04+01:00",
          "tree_id": "fb05dbbae47226c6adeec0d9c1411eea7eade017",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/1907daa0d242f6158930f2232caad60d082bd869"
        },
        "date": 1761562843367,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.5965882533050997,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=184.85795712471008, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7763.26856560826,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.304937839508057, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.34,
            "unit": "blocks/s",
            "extra": "total_wait=186s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.269956,
            "unit": "blocks^2/s^2",
            "extra": "samples=180, mean_speed=0.893889, max_speed=3.066667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.14,
            "unit": "blocks/s",
            "extra": "total_wait=218s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "3914cd32efc629d3ff49981be6391fddf05fca48",
          "message": "Merge pull request #3977 from ProvableHQ/postrelease-merge-mainnet\n\nPostrelease merge mainnet",
          "timestamp": "2025-10-28T18:25:44+01:00",
          "tree_id": "b286d4eb83ee067cb7a90065655d74aa9c0bcc73",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/3914cd32efc629d3ff49981be6391fddf05fca48"
        },
        "date": 1761674339711,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6417835426247516,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=181.6954312324524, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7984.962743467849,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.018831968307495, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.32,
            "unit": "blocks/s",
            "extra": "total_wait=189s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.100729,
            "unit": "blocks^2/s^2",
            "extra": "samples=184, mean_speed=0.845924, max_speed=2.750000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.21,
            "unit": "blocks/s",
            "extra": "total_wait=206s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "3a38e2a888ea357cb744d78c0e278be9f5a0523d",
          "message": "Merge pull request #3959 from ProvableHQ/fix/missing-block-500\n\n[Fix] Return 404 for missing blocks",
          "timestamp": "2025-10-28T13:42:18-05:00",
          "tree_id": "54efa42c9e5a998967e00c7400eff50e741c19fe",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/3a38e2a888ea357cb744d78c0e278be9f5a0523d"
        },
        "date": 1761678789114,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6666503236147463,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=180.0011031627655, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7712.427122018707,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.372869491577148, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.33,
            "unit": "blocks/s",
            "extra": "total_wait=187s, target_height=250, connect_time=23s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.220091,
            "unit": "blocks^2/s^2",
            "extra": "samples=182, mean_speed=0.851557, max_speed=3.000000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "c6093fafa3f8ba6cc351c8a988dde4ce9cbd1164",
          "message": "Merge pull request #3980 from ProvableHQ/ci/correct-cache-folder-minimal\n\n[CI] Set correct sccache folder",
          "timestamp": "2025-10-29T21:39:27+01:00",
          "tree_id": "23e092e4fc4a982f4fb4835e934fc449a9a74f39",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/c6093fafa3f8ba6cc351c8a988dde4ce9cbd1164"
        },
        "date": 1761772214789,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.596372500188444,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=184.8733184337616, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7869.181248194051,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.166241884231567, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.37,
            "unit": "blocks/s",
            "extra": "total_wait=182s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.448042,
            "unit": "blocks^2/s^2",
            "extra": "samples=177, mean_speed=0.906215, max_speed=3.216667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.2,
            "unit": "blocks/s",
            "extra": "total_wait=208s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "95d0386d8dcba0a50e277c0bacd68b9a34017d03",
          "message": "Merge pull request #3987 from ProvableHQ/update_snarkvm\n\nUpdate snarkvm",
          "timestamp": "2025-11-03T11:25:43+01:00",
          "tree_id": "7c213bbc0753d818867f2a6d6277485979fa980a",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/95d0386d8dcba0a50e277c0bacd68b9a34017d03"
        },
        "date": 1762167591353,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.674084659425659,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=179.50067448616028, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7923.783413581645,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.096187114715576, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.35,
            "unit": "blocks/s",
            "extra": "total_wait=184s, target_height=250, connect_time=23s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.135593,
            "unit": "blocks^2/s^2",
            "extra": "samples=179, mean_speed=0.828212, max_speed=2.866667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.16,
            "unit": "blocks/s",
            "extra": "total_wait=214s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "6d725c935696e9a0e7b805be968ce8547ce6b741",
          "message": "Merge pull request #3988 from ProvableHQ/trusted_peers_only\n\nAdd trusted_peers_only flag, deprecate allow_external_peers and rotate_external_peers",
          "timestamp": "2025-11-04T11:30:59+01:00",
          "tree_id": "834374b932659916302306f673afd5cce602ca81",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/6d725c935696e9a0e7b805be968ce8547ce6b741"
        },
        "date": 1762254068427,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6430232513656855,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=181.61020708084106, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7855.367213438822,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.184119701385498, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.34,
            "unit": "blocks/s",
            "extra": "total_wait=186s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.164465,
            "unit": "blocks^2/s^2",
            "extra": "samples=181, mean_speed=0.848435, max_speed=2.916667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.21,
            "unit": "blocks/s",
            "extra": "total_wait=206s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "52b46c9c32d84e163b5884a48f0710f3baabe90a",
          "message": "Merge pull request #3993 from ProvableHQ/fix_snarkos_developer\n\nFix DEFAULT_ENDPOINT in snarkos developer",
          "timestamp": "2025-11-04T17:39:10-05:00",
          "tree_id": "70620f3642eb43c826e6c985304bbfe9736f7aa7",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/52b46c9c32d84e163b5884a48f0710f3baabe90a"
        },
        "date": 1762297771390,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.546565361343023,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=188.489173412323, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7789.955201021276,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.269635438919067, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.38,
            "unit": "blocks/s",
            "extra": "total_wait=180s, target_height=250, connect_time=19s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.301409,
            "unit": "blocks^2/s^2",
            "extra": "samples=175, mean_speed=0.828381, max_speed=3.233333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "a52494661e809a6aa13652f66c1c7e663c9fccfb",
          "message": "Merge pull request #3997 from ProvableHQ/prerelease-v4.4.8\n\nUpdate snarkVM rev for v4.4.0 release",
          "timestamp": "2025-11-06T08:51:56+01:00",
          "tree_id": "5b0d680ed78ce4a733a736572367c7566c410134",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/a52494661e809a6aa13652f66c1c7e663c9fccfb"
        },
        "date": 1762417478898,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.520416347078961,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=190.44472575187683, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 8050.072659403253,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=9.937798500061035, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.36,
            "unit": "blocks/s",
            "extra": "total_wait=183s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.282855,
            "unit": "blocks^2/s^2",
            "extra": "samples=178, mean_speed=0.879869, max_speed=3.000000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "9dff2b8721ee71e353fa2f3eb01256363813d2ea",
          "message": "Merge pull request #3994 from meddle0x53/staging\n\nImplement reporting unit test timings in CI",
          "timestamp": "2025-11-06T19:48:38+01:00",
          "tree_id": "84c79ba6285e044d4430361bd88c421b3faad447",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/9dff2b8721ee71e353fa2f3eb01256363813d2ea"
        },
        "date": 1762456744388,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.704024906257137,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=177.51315784454346, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7919.6495176543785,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.101457118988037, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.32,
            "unit": "blocks/s",
            "extra": "total_wait=188s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.250978,
            "unit": "blocks^2/s^2",
            "extra": "samples=183, mean_speed=0.870583, max_speed=2.950000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "ad01a306e7bfcc9ad6d7d25074b7f6b4e58c126a",
          "message": "Merge pull request #4001 from ProvableHQ/prerelease-v4.4.9\n\nDisable peer banning",
          "timestamp": "2025-11-07T12:51:43+01:00",
          "tree_id": "dddf3bccc328caf2e901508c41fe3972960e1a99",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/ad01a306e7bfcc9ad6d7d25074b7f6b4e58c126a"
        },
        "date": 1762518188869,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.557913606089906,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=187.65293669700623, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7955.594347489505,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.055816888809204, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.36,
            "unit": "blocks/s",
            "extra": "total_wait=183s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.41021,
            "unit": "blocks^2/s^2",
            "extra": "samples=178, mean_speed=0.876966, max_speed=3.166667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.24,
            "unit": "blocks/s",
            "extra": "total_wait=201s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
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
          "id": "0f7d2f9749b097f36c0d9803c64b267f56ed09e9",
          "message": "Merge pull request #3964 from ljedrz/feat/replace_vm_locks_with_channel\n\n[Feat] Enable sequential storage processing enforcement in snarkVM",
          "timestamp": "2025-11-08T08:44:16+01:00",
          "tree_id": "1ce55b65facc2497492063f0d9b2646df09f71e8",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/0f7d2f9749b097f36c0d9803c64b267f56ed09e9"
        },
        "date": 1762589800389,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.685971711941848,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=178.70627522468567, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7844.693068236589,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.197977066040039, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.3,
            "unit": "blocks/s",
            "extra": "total_wait=192s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.233633,
            "unit": "blocks^2/s^2",
            "extra": "samples=187, mean_speed=0.877986, max_speed=2.866667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=202s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.2,
            "unit": "blocks/s",
            "extra": "total_wait=208s, target_height=250"
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
          "id": "3344448493250211147e89f3a03dfe4b62d78bbb",
          "message": "Merge pull request #4005 from ProvableHQ/log/bft-errors\n\n[Logs] Print detailed errors in BFT",
          "timestamp": "2025-11-11T20:07:04-05:00",
          "tree_id": "a39753a7794553f9f9a156daa33b39499bbe46b2",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/3344448493250211147e89f3a03dfe4b62d78bbb"
        },
        "date": 1762911601197,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6089558878984795,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=183.98164653778076, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7889.677423238288,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.13983154296875, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.35,
            "unit": "blocks/s",
            "extra": "total_wait=185s, target_height=250, connect_time=23s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.328471,
            "unit": "blocks^2/s^2",
            "extra": "samples=181, mean_speed=0.881584, max_speed=3.083333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=210s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.2,
            "unit": "blocks/s",
            "extra": "total_wait=208s, target_height=250"
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
          "id": "de5dd901bab71afba2977314cdc4a0a70f5c564b",
          "message": "Merge pull request #4006 from ProvableHQ/fix/sha-length\n\n[Fix] Use array for handshake commit hashes",
          "timestamp": "2025-11-11T20:07:44-05:00",
          "tree_id": "86432cf26f7895d9cfff8ce8a9faf1a45bc44fe2",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/de5dd901bab71afba2977314cdc4a0a70f5c564b"
        },
        "date": 1762911680547,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.560637482923166,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=187.45332098007202, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7763.691039894085,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.30437707901001, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.32,
            "unit": "blocks/s",
            "extra": "total_wait=188s, target_height=250, connect_time=20s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.182646,
            "unit": "blocks^2/s^2",
            "extra": "samples=183, mean_speed=0.841803, max_speed=2.916667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=209s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=209s, target_height=250"
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
          "id": "f8d4731528aa805844c4aa82a9d04848f71d044e",
          "message": "Merge pull request #4009 from mikenike360/staging\n\nDocument optional CUDA install path and caveats for provers",
          "timestamp": "2025-11-12T10:44:44+01:00",
          "tree_id": "7df568756599709bfb604f7d15273f0da3adf630",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/f8d4731528aa805844c4aa82a9d04848f71d044e"
        },
        "date": 1762942518227,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.7246257676422685,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=176.17098307609558, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 8106.549330582899,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=9.868563890457153, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.36,
            "unit": "blocks/s",
            "extra": "total_wait=183s, target_height=250, connect_time=23s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.123205,
            "unit": "blocks^2/s^2",
            "extra": "samples=178, mean_speed=0.821348, max_speed=3.000000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.24,
            "unit": "blocks/s",
            "extra": "total_wait=201s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=209s, target_height=250"
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
          "id": "fca0631d7516d872bcf44da81080b968e066b711",
          "message": "Merge pull request #3989 from ProvableHQ/disallow_unknown_peers_as_validator\n\nDisallow unknown peers as validator",
          "timestamp": "2025-11-12T11:01:32+01:00",
          "tree_id": "fdc78c422b220519c527eae104f96d9ad4211beb",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/fca0631d7516d872bcf44da81080b968e066b711"
        },
        "date": 1762943625720,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.5581578629866235,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=187.63501930236816, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7837.165812384394,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.20777177810669, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.4,
            "unit": "blocks/s",
            "extra": "total_wait=178s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.20763,
            "unit": "blocks^2/s^2",
            "extra": "samples=173, mean_speed=0.796146, max_speed=3.216667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=209s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=210s, target_height=250"
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
          "id": "f16a4f8e50c421f502ff0326a6d7c4b870fc31c9",
          "message": "Merge pull request #4010 from ProvableHQ/comment_pr_template_intro\n\nComment out PR template intro",
          "timestamp": "2025-11-12T12:20:34-05:00",
          "tree_id": "9e5371ab86a3b1356d167b514753cb06c01a9a4e",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/f16a4f8e50c421f502ff0326a6d7c4b870fc31c9"
        },
        "date": 1762969942342,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6604966063452027,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=180.4174449443817, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7908.107438012634,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.11620044708252, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.38,
            "unit": "blocks/s",
            "extra": "total_wait=181s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.250986,
            "unit": "blocks^2/s^2",
            "extra": "samples=176, mean_speed=0.852367, max_speed=3.166667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.21,
            "unit": "blocks/s",
            "extra": "total_wait=205s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=209s, target_height=250"
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
          "id": "b327259203d93dc09f3dbe677dc20bdfaec45afd",
          "message": "Merge pull request #4007 from ProvableHQ/refactor/sequential-ops-drops\n\n[Refactor] simplify shutting down the sequential ops thread",
          "timestamp": "2025-11-12T12:19:44-05:00",
          "tree_id": "6b87ed9ae37faf82cf7c5937b63aaad343818eab",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/b327259203d93dc09f3dbe677dc20bdfaec45afd"
        },
        "date": 1762970128662,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6225828997728633,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=183.02567291259766, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7924.137082169745,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.095736503601074, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.32,
            "unit": "blocks/s",
            "extra": "total_wait=188s, target_height=250, connect_time=21s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.152055,
            "unit": "blocks^2/s^2",
            "extra": "samples=183, mean_speed=0.863752, max_speed=2.916667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=209s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=209s, target_height=250"
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
          "id": "d2e3d09a4919e6bbef9d08ea1049a81357ff2e36",
          "message": "Merge pull request #3981 from ProvableHQ/fix/duplicate-signature\n\n[Logs] Don't show duplicate signatures as warnings",
          "timestamp": "2025-11-13T18:09:06-05:00",
          "tree_id": "110253890043d992ceb83fd10fd4b28232935a29",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/d2e3d09a4919e6bbef9d08ea1049a81357ff2e36"
        },
        "date": 1763077208982,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.5417329716640986,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=188.84753251075745, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7757.936627208685,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.312020301818848, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.38,
            "unit": "blocks/s",
            "extra": "total_wait=181s, target_height=250, connect_time=22s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.316809,
            "unit": "blocks^2/s^2",
            "extra": "samples=176, mean_speed=0.866288, max_speed=3.033333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.25,
            "unit": "blocks/s",
            "extra": "total_wait=199s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.2,
            "unit": "blocks/s",
            "extra": "total_wait=207s, target_height=250"
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
          "id": "45dee2e3995961e7d69b16c5350f33f2f66a7de6",
          "message": "Merge pull request #3923 from ProvableHQ/perf/block-caching\n\n[Perf] Use snarkVM's block caching",
          "timestamp": "2025-11-15T13:03:30+01:00",
          "tree_id": "03286ac595b0c013cce0ec33a1cc2deefcd63055",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/45dee2e3995961e7d69b16c5350f33f2f66a7de6"
        },
        "date": 1763210136450,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.6929420181711374,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=178.24371886253357, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 8068.059362397802,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=9.915643453598022, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.38,
            "unit": "blocks/s",
            "extra": "total_wait=180s, target_height=250, connect_time=15s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.447611,
            "unit": "blocks^2/s^2",
            "extra": "samples=176, mean_speed=0.896496, max_speed=3.316667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.2,
            "unit": "blocks/s",
            "extra": "total_wait=208s, target_height=250"
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
          "id": "9b4fc7605e23266e80aab48a7e0fad9ee4aeb994",
          "message": "Merge pull request #3876 from ProvableHQ/ci/bench-get-recent-block\n\n[CI] Measure speed of retrieving the most recent block",
          "timestamp": "2025-11-19T12:58:56-05:00",
          "tree_id": "fb455f14e809a82658e22b4acffa89f5b1151634",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/9b4fc7605e23266e80aab48a7e0fad9ee4aeb994"
        },
        "date": 1763577131517,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.651063881859578,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=181.0593864917755, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7943.194556711582,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.07151460647583, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-get-latest-block",
            "value": 1026.4313844011601,
            "unit": "ops/s",
            "extra": "num_ops=800, total_wait=0.7793993949890137, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.38,
            "unit": "blocks/s",
            "extra": "total_wait=180s, target_height=250, connect_time=14s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.329362,
            "unit": "blocks^2/s^2",
            "extra": "samples=175, mean_speed=0.906762, max_speed=3.166667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.28,
            "unit": "blocks/s",
            "extra": "total_wait=195s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.2,
            "unit": "blocks/s",
            "extra": "total_wait=208s, target_height=250"
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
          "id": "31c37a4755dc1c86aa99c122dcfa9916aa125577",
          "message": "Merge pull request #4019 from ljedrz/fix/full_serial_feature\n\n[Build] Fix the scope of the \"serial\" feature",
          "timestamp": "2025-11-20T12:44:10-05:00",
          "tree_id": "e8894e37ad11dc3627a4b173162caf2b8cdd5236",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/31c37a4755dc1c86aa99c122dcfa9916aa125577"
        },
        "date": 1763662442103,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.8744012353329973,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=166.99130034446716, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7764.873925054126,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.302807331085205, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-get-latest-block",
            "value": 988.9538279984038,
            "unit": "ops/s",
            "extra": "num_ops=800, total_wait=0.8089356422424316, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.42,
            "unit": "blocks/s",
            "extra": "total_wait=176s, target_height=250, connect_time=16s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.515373,
            "unit": "blocks^2/s^2",
            "extra": "samples=171, mean_speed=0.893567, max_speed=3.416667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.31,
            "unit": "blocks/s",
            "extra": "total_wait=190s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.2,
            "unit": "blocks/s",
            "extra": "total_wait=208s, target_height=250"
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
          "id": "0dfa53716471dc470a4375a7f74c665e612c2d35",
          "message": "Merge pull request #4015 from ProvableHQ/log/duplicate-deployment\n\n[Logs] Avoid `transaction/solution already exists` warnings",
          "timestamp": "2025-11-21T09:31:34+01:00",
          "tree_id": "cd445798ba880f09eeb8edb067165e86e806253d",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/0dfa53716471dc470a4375a7f74c665e612c2d35"
        },
        "date": 1763715686186,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.7063737176043814,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=177.35909748077393, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7707.903750071785,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.37895679473877, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-get-latest-block",
            "value": 948.2815902800269,
            "unit": "ops/s",
            "extra": "num_ops=800, total_wait=0.8436312675476074, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.46,
            "unit": "blocks/s",
            "extra": "total_wait=171s, target_height=250, connect_time=15s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.53218,
            "unit": "blocks^2/s^2",
            "extra": "samples=166, mean_speed=0.883032, max_speed=3.500000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.23,
            "unit": "blocks/s",
            "extra": "total_wait=203s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=209s, target_height=250"
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
          "id": "3f830442e1962f5840746958163c363cd8a8b936",
          "message": "Merge pull request #4020 from ProvableHQ/build/cargo_audit_number_prefix\n\n[Build] Add `cargo audit` exception for unmaintained `number_prefix` crate",
          "timestamp": "2025-11-21T10:48:04+01:00",
          "tree_id": "25ab887479ff01226d6c39b2cab49a592a187154",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/3f830442e1962f5840746958163c363cd8a8b936"
        },
        "date": 1763720279995,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.743621428726103,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=174.9512505531311, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7807.058458814737,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.247137308120728, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-get-latest-block",
            "value": 998.029249250982,
            "unit": "ops/s",
            "extra": "num_ops=800, total_wait=0.8015797138214111, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.42,
            "unit": "blocks/s",
            "extra": "total_wait=176s, target_height=250, connect_time=15s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.149294,
            "unit": "blocks^2/s^2",
            "extra": "samples=172, mean_speed=0.803973, max_speed=3.000000, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.18,
            "unit": "blocks/s",
            "extra": "total_wait=211s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.18,
            "unit": "blocks/s",
            "extra": "total_wait=211s, target_height=250"
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
          "id": "1bb659273341c24eec583025f16280d4580b7779",
          "message": "Merge pull request #4021 from ljedrz/fix/drop_node_on_shutdown\n\n[Fix] Drop the Node when shutting down",
          "timestamp": "2025-11-24T17:28:28-05:00",
          "tree_id": "c4bafe17fc3de2bd3369733b97f863056cfffde2",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/1bb659273341c24eec583025f16280d4580b7779"
        },
        "date": 1764025210706,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.722181095629468,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=176.32919454574585, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7899.790491325845,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.126850843429565, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-get-latest-block",
            "value": 979.4029145866225,
            "unit": "ops/s",
            "extra": "num_ops=800, total_wait=0.816824197769165, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.32,
            "unit": "blocks/s",
            "extra": "total_wait=189s, target_height=250, connect_time=14s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.243498,
            "unit": "blocks^2/s^2",
            "extra": "samples=183, mean_speed=0.844718, max_speed=2.916667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=210s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=209s, target_height=250"
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
          "id": "e77278cffd691d3d4b252df9ee669c9cea0b3b62",
          "message": "Merge pull request #4014 from ProvableHQ/log_unconnected_stake\n\nImprove logging: log stake of unconnected validators",
          "timestamp": "2025-11-24T17:38:54-05:00",
          "tree_id": "ac66e7eb8a8955f96f2ce91ca71c53ebe3d31502",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/e77278cffd691d3d4b252df9ee669c9cea0b3b62"
        },
        "date": 1764025795015,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.8277424219718337,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=169.74671959877014, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7552.421625784014,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.592628955841064, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-get-latest-block",
            "value": 1015.5487672926928,
            "unit": "ops/s",
            "extra": "num_ops=800, total_wait=0.7877514362335205, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.39,
            "unit": "blocks/s",
            "extra": "total_wait=179s, target_height=250, connect_time=16s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.169628,
            "unit": "blocks^2/s^2",
            "extra": "samples=175, mean_speed=0.855524, max_speed=2.983333, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.22,
            "unit": "blocks/s",
            "extra": "total_wait=204s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.2,
            "unit": "blocks/s",
            "extra": "total_wait=208s, target_height=250"
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
          "id": "e5b13117b713020712a988fb1226b48bf5fd1957",
          "message": "Merge pull request #4024 from ProvableHQ/test/commit-via-link\n\n[Test] Add tests for committing multiple leader certificates",
          "timestamp": "2025-11-26T12:25:06-05:00",
          "tree_id": "461904306e1498b97612f2afeecf1acff58ce539",
          "url": "https://github.com/ProvableHQ/snarkOS/commit/e5b13117b713020712a988fb1226b48bf5fd1957"
        },
        "date": 1764179879187,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "rest-get-block",
            "value": 2.7439867445628265,
            "unit": "ops/s",
            "extra": "num_ops=480, total_wait=174.92795872688293, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-block-height",
            "value": 7806.8366757924405,
            "unit": "ops/s",
            "extra": "num_ops=80000, total_wait=10.24742841720581, endpoint=http://localhost:3030/v2/testnet/block/height/latest, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "rest-get-latest-block",
            "value": 1015.8057878448175,
            "unit": "ops/s",
            "extra": "num_ops=800, total_wait=0.7875521183013916, endpoint=http://localhost:3030/v2/testnet/block, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync",
            "value": 1.38,
            "unit": "blocks/s",
            "extra": "total_wait=181s, target_height=250, connect_time=14s, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "p2p-sync-speed-variance",
            "value": 1.268142,
            "unit": "blocks^2/s^2",
            "extra": "samples=176, mean_speed=0.882576, max_speed=2.966667, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "bft-sync",
            "value": 1.17,
            "unit": "blocks/s",
            "extra": "total_wait=212s, target_height=250, connect_time=0, branch=staging, num_validators=40, git_commit=9ec2291c57, snapshot_height=250"
          },
          {
            "name": "cdn-sync",
            "value": 1.19,
            "unit": "blocks/s",
            "extra": "total_wait=209s, target_height=250"
          }
        ]
      }
    ]
  }
}
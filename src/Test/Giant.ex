// ============================================
// GIANT DEEP LEVEL ACCESS TESTS (CORRECTED)
// ============================================

// Test 25: Ultra deep mixed nesting (dict -> array -> dict -> axis -> dict -> array -> dict)
kprint "=== Giant Test 1: Ultra Deep Mixed Nesting ==="
giant1 = [&d,
    "level1": [&l,
        [&d,
            "level2": [&a,
                [&d,
                    "level3": [&l,
                        [&d,
                            "level4": [&d,
                                "msg": "DEEP_OK",
                                "nums": [&l, 11, 22, 33, 44],
                                "axis": [&a, 100, 200, 300]
                            ]
                        ],
                        "tail"
                    ]
                ],
                [&d, "dummy": 999]
            ]
        ],
        "ignored"
    ]
]

kprint giant1["level1"][0]["level2"][0]["level3"][0]["level4"]["msg"]
kprint giant1["level1"][0]["level2"][0]["level3"][0]["level4"]["nums"][2]
kprint giant1["level1"][0]["level2"][0]["level3"][0]["level4"]["axis"][-1]


// Test 26: Deep dict keys using expressions + nested access
kprint "=== Giant Test 2: Deep Dict Keys Using Expressions ==="
ab = 5
bc = 9
keySum = ab + bc      // 14
keyMul = ab * bc      // 45

giant2 = [&d,
    keySum: [&d,
        "inner": [&l,
            [&d,
                keyMul: [&a,
                    "zero",
                    [&d, "final": "SUM_MUL_OK"],
                    "two"
                ]
            ]
        ]
    ]
]

kprint giant2[14]["inner"][0][45][1]["final"]


// Test 27: Deep arrays with dict jump points + negative indexing
kprint "=== Giant Test 3: Deep Arrays + Dict + Negative Index ==="
giant3 = [&l,
    "x",
    [&d,
        "path": [&l,
            [&l,
                [&d,
                    "go": [&l, 10, 20, 30, [&d, "ok": "NEG_OK"]],
                    "axis": [&a, 1, 2, 3, 4, 5]
                ]
            ]
        ]
    ],
    "y"
]

kprint giant3[1]["path"][0][0]["go"][-1]["ok"]
kprint giant3[1]["path"][0][0]["axis"][-2]


// Test 28: Axis -> array -> axis -> dict -> array (very mixed)
kprint "=== Giant Test 4: Axis -> Array -> Axis -> Dict -> Array ==="
giant4 = [&a,
    [&l,
        [&a,
            [&d,
                "payload": [&l,
                    [&d, "value": "AXIS_MIX_OK"],
                    [&d, "n": 777]
                ]
            ],
            "skip"
        ],
        "skip2"
    ],
    "end"
]

kprint giant4[0][0][0]["payload"][0]["value"]
kprint giant4[0][0][0]["payload"][1]["n"]


// Test 29: Dynamic indices through variables (deep)
kprint "=== Giant Test 5: Dynamic Indices Deep ==="
i0 = 0
i1 = 1
i2 = 2

giant5 = [&d,
    "root": [&l,
        [&d,
            "arr": [&l,
                [&d, "x": [&a, 99, 88, 77]],
                [&d, "x": [&a, 66, 55, 44]],
                [&d, "x": [&a, 33, 22, 11]]
            ]
        ]
    ]
]

kprint giant5["root"][i0]["arr"][i2]["x"][i1]
kprint giant5["root"][0]["arr"][0]["x"][-1]


// Test 30: Very deep “company-like” structure but 2x deeper than previous
kprint "=== Giant Test 6: Deep Company Structure 2x ==="
giantCompany = [&d,
    "name": "MegaCorp",
    "meta": [&d,
        "regions": [&l,
            [&d,
                "name": "APAC",
                "countries": [&l,
                    [&d,
                        "name": "India",
                        "cities": [&l,
                            [&d,
                                "name": "Delhi",
                                "offices": [&l,
                                    [&d,
                                        "name": "HQ",
                                        "teams": [&l,
                                            [&d,
                                                "name": "Compiler",
                                                "stack": [&d,
                                                    "langs": [&l, "Rust", "C", "JS"],
                                                    "build": [&d,
                                                        "pipeline": [&l,
                                                            [&d, "step": "scan"],
                                                            [&d, "step": "parse"],
                                                            [&d, "step": "codegen"],
                                                            [&d, "step": "run", "status": "OK_DEEP"]
                                                        ]
                                                    ]
                                                ]
                                            ]
                                        ]
                                    ]
                                ]
                            ]
                        ]
                    ]
                ]
            ]
        ]
    ]
]

kprint giantCompany["meta"]["regions"][0]["countries"][0]["cities"][0]["offices"][0]["teams"][0]["stack"]["langs"][0]
kprint giantCompany["meta"]["regions"][0]["countries"][0]["cities"][0]["offices"][0]["teams"][0]["stack"]["build"]["pipeline"][3]["status"]


// Test 31: Edge but deep: dict -> dict -> array -> axis -> dict -> dict -> array -> dict
kprint "=== Giant Test 7: Edge Deep Mixed ==="
giantEdge = [&d,
    "a": [&d,
        "b": [&l,
            [&a,
                [&d,
                    "c": [&d,
                        "d": [&l,
                            [&d,
                                "e": "EDGE_DEEP_OK"
                            ]
                        ]
                    ]
                ]
            ]
        ]
    ]
]

kprint giantEdge["a"]["b"][0][0]["c"]["d"][0]["e"]

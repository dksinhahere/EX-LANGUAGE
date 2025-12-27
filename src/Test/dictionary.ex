// ============================================
// DICTIONARY TESTS
// ============================================

// Test 1: Basic dictionary with different key types
kprint "=== Dictionary Test 1: Basic Creation ==="
dict1 = [&d,
    "name": "Alice",
    "age": 25,
    "active": true
]
kprint dict1["name"]
kprint dict1["age"]
kprint dict1["active"]

// Test 2: Integer keys
kprint "=== Dictionary Test 2: Integer Keys ==="
dict2 = [&d,
    1: "one",
    2: "two",
    3: "three"
]
kprint dict2[1]
kprint dict2[2]
kprint dict2[3]

// Test 3: Mixed key types
kprint "=== Dictionary Test 3: Mixed Keys ==="
dict3 = [&d,
    "string_key": 100,
    42: "number_key",
    true: "bool_key"
]
kprint dict3["string_key"]
kprint dict3[42]
kprint dict3[true]

// Test 4: Nested dictionaries
kprint "=== Dictionary Test 4: Nested Dictionaries ==="
person = [&d,
    "name": "Bob",
    "address": [&d,
        "city": "New York",
        "zip": 10001,
        "country": [&d,
            "name": "USA",
            "code": "US"
        ]
    ]
]
kprint person["name"]
kprint person["address"]["city"]
kprint person["address"]["zip"]
kprint person["address"]["country"]["name"]
kprint person["address"]["country"]["code"]

// Test 5: Dictionary with array values
kprint "=== Dictionary Test 5: Dict with Array Values ==="
data = [&d,
    "numbers": [&l, 1, 2, 3, 4, 5],
    "names": [&l, "Alice", "Bob", "Charlie"]
]
kprint data["numbers"][0]
kprint data["numbers"][4]
kprint data["names"][0]
kprint data["names"][2]

// Test 6: Expressions as keys
kprint "=== Dictionary Test 6: Expression Keys ==="
num1 = 10
num2 = 20
dict4 = [&d,
    num1: "value_x",
    num2: "value_y",
    num1 + num2: "sum"
]
kprint dict4[10]
kprint dict4[20]
kprint dict4[30]

// ============================================
// ARRAY TESTS
// ============================================

// Test 7: Basic array
kprint "=== Array Test 1: Basic Creation ==="
arr1 = [&l, 10, 20, 30, 40, 50]
kprint arr1[0]
kprint arr1[2]
kprint arr1[4]

// Test 8: Negative indexing
kprint "=== Array Test 2: Negative Indexing ==="
arr2 = [&l, 1, 2, 3, 4, 5]
kprint arr2[-1]
kprint arr2[-2]
kprint arr2[-5]

// Test 9: Array with different types
kprint "=== Array Test 3: Mixed Types ==="
arr3 = [&l, 42, "hello", true, 3.14, 'x']
kprint arr3[0]
kprint arr3[1]
kprint arr3[2]
kprint arr3[3]
kprint arr3[4]

// Test 10: Nested arrays
kprint "=== Array Test 4: Nested Arrays ==="
matrix = [&l,
    [&l, 1, 2, 3],
    [&l, 4, 5, 6],
    [&l, 7, 8, 9]
]
kprint matrix[0][0]
kprint matrix[0][2]
kprint matrix[1][1]
kprint matrix[2][2]

// Test 11: 3D array
kprint "=== Array Test 5: 3D Array ==="
cube = [&l,
    [&l,
        [&l, 1, 2],
        [&l, 3, 4]
    ],
    [&l,
        [&l, 5, 6],
        [&l, 7, 8]
    ]
]
kprint cube[0][0][0]
kprint cube[0][1][1]
kprint cube[1][0][1]
kprint cube[1][1][1]

// Test 12: Array with expressions
kprint "=== Array Test 6: Expression Elements ==="
val1 = 5
val2 = 10
arr4 = [&l, val1, val2, val1 + val2, val1 * val2, val1 - val2]
kprint arr4[0]
kprint arr4[2]
kprint arr4[3]

// Test 13: Array in dictionary
kprint "=== Array Test 7: Arrays in Dict ==="
scores = [&d,
    "math": [&l, 90, 85, 95],
    "science": [&l, 88, 92, 87]
]
kprint scores["math"][0]
kprint scores["math"][2]
kprint scores["science"][1]

// ============================================
// AXIS TESTS (Const Vector)
// ============================================

// Test 14: Basic axis
kprint "=== Axis Test 1: Basic Creation ==="
axis1 = [&a, 10, 20, 30, 40, 50]
kprint axis1[0]
kprint axis1[2]
kprint axis1[4]

// Test 15: Negative indexing on axis
kprint "=== Axis Test 2: Negative Indexing ==="
axis2 = [&a, 100, 200, 300, 400, 500]
kprint axis2[-1]
kprint axis2[-3]
kprint axis2[-5]

// Test 16: Axis with expressions
kprint "=== Axis Test 3: Expression Elements ==="
var1 = 7
var2 = 3
axis3 = [&a, var1, var2, var1 + var2, var1 * var2, var1 - var2]
kprint axis3[0]
kprint axis3[2]
kprint axis3[4]

// Test 17: Nested axis
kprint "=== Axis Test 4: Nested Axis ==="
points = [&a,
    [&a, 1, 2, 3],
    [&a, 4, 5, 6],
    [&a, 7, 8, 9]
]
kprint points[0][0]
kprint points[1][1]
kprint points[2][2]

// Test 18: Axis with mixed types
kprint "=== Axis Test 5: Mixed Types ==="
axis4 = [&a, 1, 2.5, "text", true, 'c']
kprint axis4[0]
kprint axis4[1]
kprint axis4[2]
kprint axis4[3]
kprint axis4[4]

// ============================================
// MIXED TESTS
// ============================================

// Test 19: Dict with arrays and axes
kprint "=== Mixed Test 1: Dict with Arrays and Axes ==="
complex = [&d,
    "array": [&l, 1, 2, 3],
    "axis": [&a, 4, 5, 6],
    "nested": [&d,
        "inner_array": [&l, 7, 8, 9]
    ]
]
kprint complex["array"][0]
kprint complex["axis"][1]
kprint complex["nested"]["inner_array"][2]

// Test 20: Array of dictionaries
kprint "=== Mixed Test 2: Array of Dicts ==="
users = [&l,
    [&d, "name": "Alice", "age": 25],
    [&d, "name": "Bob", "age": 30],
    [&d, "name": "Charlie", "age": 35]
]
kprint users[0]["name"]
kprint users[0]["age"]
kprint users[1]["name"]
kprint users[2]["age"]

// Test 21: Dictionary of dictionaries
kprint "=== Mixed Test 3: Dict of Dicts ==="
config = [&d,
    "database": [&d,
        "host": "localhost",
        "port": 5432
    ],
    "cache": [&d,
        "host": "redis",
        "port": 6379
    ]
]
kprint config["database"]["host"]
kprint config["database"]["port"]
kprint config["cache"]["host"]
kprint config["cache"]["port"]

// Test 22: Using dynamic range with arrays
kprint "=== Mixed Test 4: Dynamic Range ==="
range_arr = ::[1..5]
kprint range_arr[0]
kprint range_arr[4]

// Test 23: Complex nested structure
kprint "=== Mixed Test 5: Complex Structure ==="
company = [&d,
    "name": "TechCorp",
    "departments": [&l,
        [&d,
            "name": "Engineering",
            "employees": [&l, "Alice", "Bob", "Charlie"]
        ],
        [&d,
            "name": "Sales",
            "employees": [&l, "Dave", "Eve"]
        ]
    ]
]
kprint company["name"]
kprint company["departments"][0]["name"]
kprint company["departments"][0]["employees"][0]
kprint company["departments"][1]["employees"][1]

// Test 24: Edge cases
kprint "=== Edge Test 1: Empty Structures ==="
empty_dict = [&d, "name":[&d, "age": [&l, 1, 2, 3, 4]]]
kprint empty_dict["name"]["age"][2]
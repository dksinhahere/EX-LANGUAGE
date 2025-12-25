// ==========================================
// ARRAY (LIST) TEST CASES
// ==========================================

// Test Case 1: Empty array
empty_list = []

// Test Case 2: Simple array with numbers
numbers = [1, 2, 3, 4, 5]

// Test Case 3: Array with mixed types
mixed = [1, "hello", true, 3.14, nil]

// Test Case 4: Array with expressions
calculated = [1 + 1, 2 * 3, 10 / 2]

// Test Case 5: Nested arrays
matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]

// Test Case 6: Array indexing
first = numbers[0]
second = numbers[1]
last = numbers[4]

// Test Case 7: Array with function label
results = [log(src="test"), calculate(src=10), getValue(src=Nil)]

// Test Case 8: Modifying array elements
numbers[0] = 10
numbers[2] = numbers[1] + 5

// Test Case 9: Array with variables
x = 10
y = 20
coords = [x, y, x + y]

// Test Case 10: Multi-dimensional array access
value = matrix[0][1]
matrix[1][2] = 99

// ==========================================
// DICTIONARY (DICT) TEST CASES
// ==========================================

// Test Case 11: Empty dictionary
empty_dict = {}

// Test Case 12: Simple dictionary with string keys
person = {"name": "Danish", "age": 25, "city": "Patna"}

// Test Case 13: Dictionary with mixed key types
mixed_dict = {1: "one", "two": 2, true: "yes"}

// Test Case 14: Dictionary with computed values
config = {
    "width": 1920,
    "height": 1080,
    "area": 1920 * 1080
}

// Test Case 15: Nested dictionaries
user = {
    "name": "John",
    "address": {
        "street": "123 Main St",
        "city": "New York",
        "zip": "10001"
    },
    "age": 30
}

// Test Case 16: Dictionary access
name = person["name"]
age = person["age"]

// Test Case 17: Modifying dictionary values
person["age"] = 26
person["country"] = "India"

// Test Case 18: Dictionary with array values
data = {
    "numbers": [1, 2, 3],
    "names": ["Alice", "Bob", "Charlie"],
    "flags": [true, false, true]
}

// Test Case 19: Nested access
street = user["address"]["street"]
first_number = data["numbers"][0]

// Test Case 20: Dictionary with expressions as keys
key1 = "first"
key2 = "second"
dynamic = {
    key1: 100,
    key2: 200
}

// ==========================================
// COMBINED ARRAY + DICT TEST CASES
// ==========================================

// Test Case 21: Array of dictionaries
users = [
    {"name": "Alice", "age": 25},
    {"name": "Bob", "age": 30},
    {"name": "Charlie", "age": 35}
]

// Test Case 22: Dictionary of arrays
collections = {
    "even": [2, 4, 6, 8],
    "odd": [1, 3, 5, 7],
    "primes": [2, 3, 5, 7, 11]
}

// Test Case 23: Complex nested structure
database = {
    "users": [
        {"id": 1, "name": "Alice", "scores": [95, 87, 92]},
        {"id": 2, "name": "Bob", "scores": [78, 85, 90]}
    ],
    "metadata": {
        "version": 1.0,
        "updated": "2024-01-01"
    }
}

// Test Case 24: Accessing complex nested data
alice_first_score = database["users"][0]["scores"][0]
bob_name = database["users"][1]["name"]
version = database["metadata"]["version"]

// Test Case 25: Modifying nested structures
database["users"][0]["scores"][1] = 90
database["metadata"]["updated"] = "2024-01-15"

// ==========================================
// ARRAY/DICT WITH CONTROL FLOW
// ==========================================

// Test Case 26: Using arrays in if statements
scores = [85, 90, 78]
if (scores[0] > 80) [
    log(src="Good score!")
]

// Test Case 27: Using dictionaries in if statements
settings = {"debug": true, "verbose": false}
if (settings["debug"]) [
    log(src="Debug mode enabled")
]

// Test Case 28: Iterating with labels (loop simulation)
items = [1, 2, 3, 4, 5]
index = 0

label @loop [
    if (index >= 5) [
        jump end
    ]
    else [
        log(src=items[index])
        index = index + 1
        jump loop
    ]
]

label @end [
    pass
]

// Test Case 29: Dictionary in function
label process_user(user_dict) [
    name = user_dict["name"]
    age = user_dict["age"]
    log(src=name)
    log(src=age)
]

user_data = {"name": "Test", "age": 25}
process_user(user_dict=user_data)

// Test Case 30: Returning arrays from labels
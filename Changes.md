## 0.5.1 - 2023-06-03
**Dependencies**
* Updated libdtf to 0.5.1

**Fixes coming from depency**
* It is now handled correctly when an array has multiple ocurrences of a value
* Value differences no longer print out whole arrays, just indicate there is a difference. This should improve performance.

**Test data**
* Adjusted some test data to better represent the capabilities of the tool.
* Added larger test files for performance check

## 0.5.0 - 2023-05-22
**Features:**
* Check 2 JSON files for differences
* Configure Which differences to check for
* Save Results to custom JSON file
* Read saved results and configure which data to display

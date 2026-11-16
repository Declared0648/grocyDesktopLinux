# Features and behavior

The supplied Grocy 4.7.1 source folder provides implementation evidence for the workflows below. Source locations are named by file and method because these PHP files have no native function addresses. A native decompilation was attempted, but the original MSI is currently missing and the Grocy entry point (`public/index.php`) is PHP text, not native machine code; native string/function tools cannot produce addresses for it. No addresses are invented.

## Stock and shopping rules from Grocy source

- `services/StockService.php`, `AddProduct`: stock additions must be greater than zero. When no best-before date is supplied, the product's configured default determines the date: `-1` means 2999-12-31 (never), a positive number means today plus that many days, and other values mean today. Freezer defaults can override this when enabled.
- `services/StockService.php`, `AddMissingProductsToShoppingList`: the shopping amount comes from each product's missing amount, rounded to two decimal places. An existing list entry is increased only if its amount is lower than the missing amount; it is not blindly added a second time. New entries use the product's purchase unit.
- `controllers/StockController.php`, `Overview`: the overview uses the current-stock view's `is_in_stock_or_below_min_stock` flag, with an optional setting to show all out-of-stock products.
- `services/StockService.php`, `ConsumeProduct`/stock transactions: Grocy models distinct transaction types such as purchase, consume, inventory correction, self-production, opening and transfers.

## Chores

- `services/ChoresService.php` defines hourly, daily, weekly, monthly, yearly, adaptive, and manual schedule types.
- `controllers/ChoresController.php`, `Overview`: chores are classified overdue, due today, or due soon based on their next estimated execution time and the user's configured due-soon window.
- The remake exposes Grocy's schedule choices and records basic completion. It adds household members, chore assignments, date-based overdue/today/within-seven-days reminders, and a prefilled email draft opened by the user's mail app; messages are never sent automatically. Recurring due-date calculation, execution history, and stock transactions per batch are not implemented.

## Remake behavior

- Stock and shopping amounts now support fractional values. New stock amounts must be finite and greater than zero; stock cannot be reduced below zero.
- “Add missing stock” adds `round(max(0, minimum - current stock), 2)`; an existing matching shopping item is raised to that shortfall only if it is smaller.
- Pantry, shopping, meals, chores, household members, and user-added recipes are stored in a local JSON file (browser local storage in the WebAssembly build). Older saved data without meal-plan, chore, member, or recipe fields remains readable. Recipes can be added, edited, and removed; ingredients are entered one per line as name, positive amount, and unit.
- The desktop wrapper's exact UI remains unavailable: the MSI was found empty during earlier inspection and is now missing. This Rust app does not connect to a Grocy server or implement Grocy's full batch/date history.
- `--screenshot <file.png>` leaves the window unfocused, requests 120 rendered frames, writes the screenshot as a PNG, and exits.

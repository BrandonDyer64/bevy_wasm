# Changelog

## 0.10.0

-   Upgraded to Bevy v0.10

## 0.9.9

-   Code is now dual licensed under Apache-2.0 and MIT.
-   Added [anyhow](https://crates.io/crates/anyhow)
-   Removed unwraps
-   Added GitHub Actions workflow

## 0.9.8

-   The CHANGELOG.md file
-   Mods are now entities with a `WasmMod` component.
-   Mods can now be removed
-   Mods can now be sent serialized events individually.
-   Files restructured to be easier to understand.
-   Protocol version calculation now correctly parses integers.

## 0.9.7

-   Enabled sharing resources with mods.
-   Mod startup systems are now run _after_ the first update
-   Updated cubes example to orbit around a center point

## 0.9.6

-   Fixed issues with the README

## 0.9.5

-   Protocol version checking
-   Crate version now matches the major and minor version of Bevy

1. Everything is black and white (Contrast problems)
2. Localization problems (i18 localization might be turned off)
2. Tauri command problems:
3.1. Ошибка загрузки коллекций: Command get_collections not found
3.2. Ошибка загрузки аналитики: Failed to get storage analytics: get_storage_analytics not yet implemented
3.3 Ошибка загрузки кандидатов на очистку: Failed to get cleanup candidates: get_cleanup_candidates not yet implemented
4. UI is lacking normal looks, it's all sluggy and old. I'd like round corners, sidebar to be able to collapse.
5. Search bar is not centralized.
6. Saving settings is not working because of failed settings validation.
7. App is old, not smooth, freaky and unuseful.
8. Checkmark is bigger than checkbox.
9. Debug log selector is white on white text (HOLY SHIT)
10. No detailed panel for movies open when you press on movie tile.
11. Movie tile is overall awful it's 3:2 instead of 2:3 ratio.
12. No metadata shows in UI (Posters f.e, but on detailed panel it must be descriptions, ratings, cast, duration, all things that we parse from indexer)
13. We need to implement title_normalizer for names of shows/movies on tiles. (Preferably we should use ACTUAL title_normalizer (Title_normalizer_core.rs and adjustent to him title_normalizer folder with folder common inside it (BUT IT NEEDS DEBUGGING AND TESTING)) all because current implementation is very weak compared to it)
14. OVERALL APP IS LOOKING AWFUL AND NEEDS TO STEAL DESIGN FROM NETFLIX OR OTHER STERAMING SERVICES. HEAVY WORK ON UI/UX WITH LATEST TECH IS NEEDED!

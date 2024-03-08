# DCT Backend
- [x] Admin auth
- [x] CRUD faculties
- [x] CRUD classes
- [x] CRUD classes files
- [x] Classes filter (faculty, semester)
- [ ] Add temporary student checking
- [ ] Users CRUD, check using the old_choices that the user didn't already choose this class already
- [ ] Let admins export to csv, move choices to old_choices 
- [ ] Make the student not able to pick a class from their own faculty
- [ ] Implement user auth
- [ ] Move to UVT UMS auth
- [ ] Admin features (logging, view logs, etc.)
- [ ] Change configuration to be less verbose, example -> https://www.thorsten-hans.com/working-with-environment-variables-in-rust/
- [ ] Requirements are optional, when creating a class, deserialize empty string in requirements as None

# DCT Frontend
- [x] List classes frontend with filters for semesters, faculties, it should be similar to a "listview"
- [ ] Pick classes, ask for nr matricol
- [ ] Admin panel
- [ ] UI for export csv
- [x] FE for CRUD Faculties, classes

Maybe after everything is done, cache all possible scenarios of classes filter/search and only invalidate manually when the expected db tables are changed
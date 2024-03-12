# DCT Backend
- [x] Admin auth
- [x] CRUD faculties
- [x] CRUD classes
- [x] CRUD classes files
- [x] Classes filter (faculty, semester)
- [x] Add temporary student checking
- [x] Implement student auth
- [x] Basic Logging
- [x] Add db with sample data
- [x] Choices CRUD
- [x] Make the student not able to pick a class from their own faculty
- [x] Let admins export to csv, json
- [x] Check using the old_choices that the user didn't already choose this class already in the past
- [x] Let admins move choices to old_choices 
- [ ] Move to UVT UMS auth
- [ ] Make choices optional, and add required fields, in case the DCT is already picked for students from a faculty

# DCT Frontend
- [x] List classes frontend with filters for semesters, faculties, it should be similar to a "listview"
- [ ] Admin panel
- [ ] UI for move csv
- [x] FE for CRUD Faculties, classes

Maybe add caching for export-csv and export-json.
Maybe after everything is done, cache all possible scenarios of classes filter/search and only invalidate manually when the expected db tables are changed
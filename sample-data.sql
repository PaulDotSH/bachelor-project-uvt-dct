INSERT INTO public.faculties
(id, "name")
VALUES(1, 'Facultatea de SP');
INSERT INTO public.faculties
(id, "name")
VALUES(2, 'Facultatea de FMI');
INSERT INTO public.faculties
(id, "name")
VALUES(3, 'Facultatea de ARTE');
INSERT INTO public.faculties
(id, "name")
VALUES(4, 'Facultatea de CARTE');
INSERT INTO public.faculties
(id, "name")
VALUES(5, 'Facultatea de TARTE');
INSERT INTO public.faculties
(id, "name")
VALUES(6, 'Fac. De. SC. Vietii');

INSERT INTO public.classes
(id, "name", descr, faculty, "semester", disabled, requirements, prof)
VALUES(1, 'Blockchain si dodgecoin', 'Invatam cum sa dam scam cu crypto la copii si sa ne luam rarri din banii aia', 2, 'First'::public."semester", false, NULL, 'Nu mai stiu numele');
INSERT INTO public.classes
(id, "name", descr, faculty, "semester", disabled, requirements, prof)
VALUES(2, 'Hacking deepweb', 'Invatam cum sa fim hackeri de pe deepweb', 2, 'Second'::public."semester", false, NULL, 'Cristi POPESCO');
INSERT INTO public.classes
(id, "name", descr, faculty, "semester", disabled, requirements, prof)
VALUES(3, 'Depanator PC', 'Invatam cum sa dam reset la bios, sa schimbam toate componentele pana ne dam seama ca placa de baza e problema etc', 2, 'First'::public."semester", false, NULL, 'Cristi POPESCO');
INSERT INTO public.classes
(id, "name", descr, faculty, "semester", disabled, requirements, prof)
VALUES(4, 'Totul despre masonerie', 'Invatam cum sa fim masoni', 4, 'First'::public."semester", false, NULL, 'Gigel Rothschild');
INSERT INTO public.classes
(id, "name", descr, faculty, "semester", disabled, requirements, prof)
VALUES(5, 'Cum sa gatesti', 'Cum sa gatesti mai bine decat Hino', 5, 'First'::public."semester", false, NULL, 'Gordon Ramsay');
INSERT INTO public.classes
(id, "name", descr, faculty, "semester", disabled, requirements, prof)
VALUES(6, 'Cum sa fii emoloaica', 'Tutorial pentru incepatori dar si avansati', 3, 'Second'::public."semester", false, 'Sa ai cromozomi XX', 'Nu se stie inca');
INSERT INTO public.classes
(id, "name", descr, faculty, "semester", disabled, requirements, prof)
VALUES(7, 'Cum sa fii ca Mike', 'Tutorial pentru incepatori dar si avansati', 1, 'Second'::public."semester", false, NULL, 'Mike Obama');
INSERT INTO public.classes
(id, "name", descr, faculty, "semester", disabled, requirements, prof)
VALUES(8, 'C, C++, Hacking si cum sa faci bench cu 130 de kg', 'C, C++, Hacking si cum sa faci bench cu 130 de kg, pe langa aceste lucruri invatam si istoria manelelor si a trapului romanesc', 2, 'Second'::public."semester", false, NULL, 'Paul Abrudan');
INSERT INTO public.classes
(id, "name", descr, faculty, "semester", disabled, requirements, prof)
VALUES(9, 'Cum sa conduci beat', 'Pe langa cum sa conduci beat, inveti si sa nu te prinda politia', 6, 'Second'::public."semester", false, 'Masina, de permis nu avem nevoie', 'Wizzzard');


INSERT INTO public.students
(nr_mat, email, cnp3, "token", tok_expire, faculty)
VALUES('IA2', 'student2@example.com', '234', '', '2024-03-15 16:30:05.959', 1);
INSERT INTO public.students
(nr_mat, email, cnp3, "token", tok_expire, faculty)
VALUES('IA3', 'student3@example.com', '345', '', '2024-03-15 16:30:05.959', 1);
INSERT INTO public.students
(nr_mat, email, cnp3, "token", tok_expire, faculty)
VALUES('IA4', 'student4@example.com', '456', '', '2024-03-15 16:30:05.959', 1);
INSERT INTO public.students
(nr_mat, email, cnp3, "token", tok_expire, faculty)
VALUES('IA5', 'student5@example.com', '567', '', '2024-03-15 16:30:05.959', 1);
INSERT INTO public.students
(nr_mat, email, cnp3, "token", tok_expire, faculty)
VALUES('IA1', 'student1@example.com', '123', '', '2024-03-15 21:39:43.198', 1);
INSERT INTO public.students
(nr_mat, email, cnp3, "token", tok_expire, faculty)
VALUES('111', 'paul.abrudan03@e-uvt.ro', '000', '', '2024-03-18 08:08:16.001', 2);
INSERT INTO public.students
(nr_mat, email, cnp3, "token", tok_expire, faculty)
VALUES('112', 'bogdan.anghel@e-uvt.ro', '319', '', '2024-03-18 08:08:48.538', 1);
INSERT INTO public.students
(nr_mat, email, cnp3, "token", tok_expire, faculty)
VALUES('113', 'marius.gheoghe@e-uvt.ro', '194', '', '2024-03-18 08:09:07.749', 1);
INSERT INTO public.students
(nr_mat, email, cnp3, "token", tok_expire, faculty)
VALUES('114', 'gela.blonda@e-uvt.ro', '712', '', '2024-03-18 08:10:12.703', 3);
INSERT INTO public.students
(nr_mat, email, cnp3, "token", tok_expire, faculty)
VALUES('115', 'jasmine.popa@e-uvt.ro', '152', '', '2024-03-18 08:12:44.115', 3);
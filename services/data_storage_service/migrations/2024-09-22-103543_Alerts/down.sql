-- This file should undo anything in `up.sql`
drop type IF EXISTS severity CASCADE ;
drop table IF EXISTS alert CASCADE ;
drop table IF EXISTS data_source CASCADE ;
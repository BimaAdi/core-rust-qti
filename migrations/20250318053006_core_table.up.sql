CREATE TABLE public."user" (
	id uuid NOT NULL,
	user_name varchar NOT NULL,
	"password" varchar NOT NULL,
	is_active bool NULL,
	created_by uuid NULL,
	updated_by uuid NULL,
	created_date timestamptz NULL,
	updated_date timestamptz NULL,
	deleted_date timestamptz NULL,
	is_2faenabled bool NULL,
	CONSTRAINT user_pkey PRIMARY KEY (id),
	CONSTRAINT user_created_by_fkey FOREIGN KEY (created_by) REFERENCES public."user"(id),
	CONSTRAINT user_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public."user"(id)
);
CREATE INDEX ix_user_id ON public."user" USING btree (id);
CREATE UNIQUE INDEX ix_user_user_name ON public."user" USING btree (user_name);

CREATE TABLE public.user_profile (
	id uuid NOT NULL,
	user_id uuid NULL,
	first_name varchar NULL,
	last_name varchar NULL,
	address varchar NULL,
	email varchar NULL,
	CONSTRAINT user_profile_pkey PRIMARY KEY (id),
	CONSTRAINT user_profile_user_id_fkey FOREIGN KEY (user_id) REFERENCES public."user"(id)
);
CREATE INDEX ix_user_profile_id ON public.user_profile USING btree (id);

CREATE TABLE public."role" (
	id uuid NOT NULL,
	role_name varchar NOT NULL,
	description varchar NULL,
	is_active bool NULL,
	created_by uuid NULL,
	updated_by uuid NULL,
	created_date timestamptz NULL,
	updated_date timestamptz NULL,
	deleted_date timestamptz NULL,
	CONSTRAINT role_pkey PRIMARY KEY (id),
	CONSTRAINT role_created_by_fkey FOREIGN KEY (created_by) REFERENCES public."user"(id),
	CONSTRAINT role_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public."user"(id)
);
CREATE INDEX ix_role_id ON public.role USING btree (id);
CREATE UNIQUE INDEX ix_role_role_name ON public.role USING btree (role_name);

CREATE TABLE public."group" (
	id uuid NOT NULL,
	group_name varchar NOT NULL,
	description varchar NULL,
	is_active bool NULL,
	parent_id uuid NULL,
	created_by uuid NULL,
	updated_by uuid NULL,
	created_date timestamptz NULL,
	updated_date timestamptz NULL,
	deleted_date timestamptz NULL,
	CONSTRAINT group_pkey PRIMARY KEY (id),
	CONSTRAINT group_created_by_fkey FOREIGN KEY (created_by) REFERENCES public."user"(id),
	CONSTRAINT group_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public."group"(id) ON DELETE CASCADE,
	CONSTRAINT group_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public."user"(id)
);
CREATE UNIQUE INDEX ix_group_group_name ON public."group" USING btree (group_name);
CREATE INDEX ix_group_id ON public."group" USING btree (id);

CREATE TABLE public.user_group_roles (
	id uuid NOT NULL,
	user_id uuid NULL,
	group_id uuid NULL,
	role_id uuid NULL,
	CONSTRAINT user_group_roles_pkey PRIMARY KEY (id),
	CONSTRAINT user_group_roles_group_id_fkey FOREIGN KEY (group_id) REFERENCES public."group"(id),
	CONSTRAINT user_group_roles_role_id_fkey FOREIGN KEY (role_id) REFERENCES public."role"(id),
	CONSTRAINT user_group_roles_user_id_fkey FOREIGN KEY (user_id) REFERENCES public."user"(id)
);
CREATE INDEX ix_user_group_roles_id ON public.user_group_roles USING btree (id);

CREATE TABLE public."permission" (
	id uuid NOT NULL,
	permission_name varchar NOT NULL,
	is_user bool NULL,
	is_role bool NULL,
	is_group bool NULL,
	description varchar NULL,
	created_by uuid NULL,
	updated_by uuid NULL,
	created_date timestamptz NULL,
	updated_date timestamptz NULL,
	CONSTRAINT permission_pkey PRIMARY KEY (id),
	CONSTRAINT permission_created_by_fkey FOREIGN KEY (created_by) REFERENCES public."user"(id),
	CONSTRAINT permission_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public."user"(id)
);
CREATE INDEX ix_permission_id ON public.permission USING btree (id);

CREATE TABLE public.permission_attribute (
	id uuid NOT NULL,
	"name" varchar NOT NULL,
	description varchar NULL,
	created_date timestamptz NULL,
	updated_date timestamptz NULL,
	CONSTRAINT permission_attribute_pkey PRIMARY KEY (id)
);
CREATE INDEX ix_permission_attribute_id ON public.permission_attribute USING btree (id);

CREATE TABLE public.permission_attribute_list (
	permission_id uuid NOT NULL,
	attribute_id uuid NOT NULL,
	CONSTRAINT permission_attribute_list_pkey PRIMARY KEY (permission_id, attribute_id),
	CONSTRAINT permission_attribute_list_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.permission_attribute(id) ON DELETE CASCADE ON UPDATE CASCADE,
	CONSTRAINT permission_attribute_list_permission_id_fkey FOREIGN KEY (permission_id) REFERENCES public."permission"(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE public.user_permission (
	permission_id uuid NOT NULL,
	user_id uuid NOT NULL,
	attribute_id uuid NOT NULL,
	created_by uuid NULL,
	updated_by uuid NULL,
	created_date timestamptz NULL,
	updated_date timestamptz NULL,
	CONSTRAINT user_permission_pkey PRIMARY KEY (permission_id, user_id, attribute_id),
	CONSTRAINT user_permission_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.permission_attribute(id) ON DELETE CASCADE ON UPDATE CASCADE,
	CONSTRAINT user_permission_created_by_fkey FOREIGN KEY (created_by) REFERENCES public."user"(id),
	CONSTRAINT user_permission_permission_id_fkey FOREIGN KEY (permission_id) REFERENCES public."permission"(id) ON DELETE CASCADE ON UPDATE CASCADE,
	CONSTRAINT user_permission_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public."user"(id),
	CONSTRAINT user_permission_user_id_fkey FOREIGN KEY (user_id) REFERENCES public."user"(id)
);

CREATE TABLE public.role_permissions (
	role_id uuid NOT NULL,
	permission_id uuid NOT NULL,
	attribute_id uuid NOT NULL,
	created_by uuid NULL,
	updated_by uuid NULL,
	created_date timestamptz NULL,
	updated_date timestamptz NULL,
	CONSTRAINT role_permissions_pkey PRIMARY KEY (role_id, permission_id, attribute_id),
	CONSTRAINT role_permissions_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.permission_attribute(id) ON DELETE CASCADE ON UPDATE CASCADE,
	CONSTRAINT role_permissions_created_by_fkey FOREIGN KEY (created_by) REFERENCES public."user"(id),
	CONSTRAINT role_permissions_permission_id_fkey FOREIGN KEY (permission_id) REFERENCES public."permission"(id) ON DELETE CASCADE ON UPDATE CASCADE,
	CONSTRAINT role_permissions_role_id_fkey FOREIGN KEY (role_id) REFERENCES public."role"(id) ON DELETE CASCADE ON UPDATE CASCADE,
	CONSTRAINT role_permissions_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public."user"(id)
);

CREATE TABLE public.group_permissions (
	group_id uuid NOT NULL,
	permission_id uuid NOT NULL,
	attribute_id uuid NOT NULL,
	created_by uuid NULL,
	updated_by uuid NULL,
	created_date timestamptz NULL,
	updated_date timestamptz NULL,
	CONSTRAINT group_permissions_pkey PRIMARY KEY (group_id, permission_id, attribute_id),
	CONSTRAINT group_permissions_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.permission_attribute(id) ON DELETE CASCADE ON UPDATE CASCADE,
	CONSTRAINT group_permissions_created_by_fkey FOREIGN KEY (created_by) REFERENCES public."user"(id),
	CONSTRAINT group_permissions_group_id_fkey FOREIGN KEY (group_id) REFERENCES public."group"(id) ON DELETE CASCADE ON UPDATE CASCADE,
	CONSTRAINT group_permissions_permission_id_fkey FOREIGN KEY (permission_id) REFERENCES public."permission"(id) ON DELETE CASCADE ON UPDATE CASCADE,
	CONSTRAINT group_permissions_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public."user"(id)
);

CREATE TABLE public.menu (
	id uuid NOT NULL,
	"name" varchar NOT NULL,
	description varchar NULL,
	parent_id uuid NULL,
	"order" int4 NULL,
	url varchar NULL,
	parent_only bool NULL,
	created_by uuid NULL,
	updated_by uuid NULL,
	created_date timestamptz NULL,
	updated_date timestamptz NULL,
	icon varchar NULL,
	permission_id uuid NULL,
	attribute_id uuid NULL,
	CONSTRAINT menu_name_key UNIQUE (name),
	CONSTRAINT menu_pkey PRIMARY KEY (id),
	CONSTRAINT menu_created_by_fkey FOREIGN KEY (created_by) REFERENCES public."user"(id),
	CONSTRAINT menu_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.menu(id) ON DELETE CASCADE,
	CONSTRAINT menu_permission_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.permission_attribute(id) ON DELETE SET NULL,
	CONSTRAINT menu_permission_id_fkey FOREIGN KEY (permission_id) REFERENCES public."permission"(id) ON DELETE SET NULL,
	CONSTRAINT menu_updated_by_fkey FOREIGN KEY (updated_by) REFERENCES public."user"(id)
);
CREATE INDEX ix_menu_id ON public.menu USING btree (id);

CREATE TYPE public."httpmethodenum" AS ENUM (
	'GET',
	'POST',
	'PUT',
	'PATCH',
	'DELETE',
	'OPTIONS'
);

CREATE TABLE public.api_list (
	api_path varchar NOT NULL,
	"method" public."httpmethodenum" NOT NULL,
	permission_id uuid NOT NULL,
	attribute_id uuid NOT NULL,
	CONSTRAINT api_list_pkey PRIMARY KEY (api_path, method),
	CONSTRAINT api_list_attribute_id_fkey FOREIGN KEY (attribute_id) REFERENCES public.permission_attribute(id),
	CONSTRAINT api_list_permission_id_fkey FOREIGN KEY (permission_id) REFERENCES public."permission"(id)
);
CREATE INDEX ix_api_list_api_path ON public.api_list USING btree (api_path);

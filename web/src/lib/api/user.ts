import type { User } from "@models/user";
import type { SearchParamsOption } from "ky";
import api, { api_root } from ".";

export async function getUserList(
  page?: number,
  page_size?: number,
  order?: string,
  filter?: string,
  with_institute_id?: number
) {
  return await api
    .get(`${api_root}/user`, {
      searchParams: JSON.parse(
        JSON.stringify({
          page,
          page_size,
          order,
          filter,
          with_institute_id,
        })
      ) as SearchParamsOption,
    })
    .json<[User[], number]>();
}

export async function getUser(id: number) {
  return await api.get(`${api_root}/user/${id}`).json<User>();
}

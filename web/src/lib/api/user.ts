import type { Ip } from "@models/ip";
import type { OAuth } from "@models/oauth";
import type { Team } from "@models/team";
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

export async function getUserTeams(id: number) {
  return await api.get(`${api_root}/user/${id}/team`).json<Team[]>();
}

export async function updateUser(user: User) {
  return await api.patch(`${api_root}/user/${user.id}`, { json: user }).json<User>();
}

export async function deleteUser(id: number) {
  return await api.delete(`${api_root}/user/${id}`).json();
}

export async function getUserIpList(id: number) {
  return await api.get(`${api_root}/user/${id}/ip`).json<Ip[]>();
}

export async function getUserOAuthList(id: number) {
  return await api.get(`${api_root}/user/${id}/oauth`).json<OAuth[]>();
}

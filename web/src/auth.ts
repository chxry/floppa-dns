import { createContext } from "react";

export type User = { username: string; created: string };

export type AuthContext = {
  token: string | null;
  setToken: React.Dispatch<string | null>;
  user: User | null;
  setUser: React.Dispatch<User | null>;
};

export const AuthContext = createContext<AuthContext>(undefined!);

export const fetchAuth = async (
  url: RequestInfo,
  token: string,
  opts?: RequestInit,
) => {
  return fetch(
    url,
    Object.assign({ headers: { Authorization: `Bearer ${token}` } }, opts),
  );
};

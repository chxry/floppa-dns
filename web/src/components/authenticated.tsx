import { JSX } from "solid-js";
import { Navigate } from "@solidjs/router";

import { token, user } from "..";

const Authenticated = (props: { children: JSX.Element }) => {
  return token() ? (
    user() ? (
      props.children
    ) : (
      <p>loading</p>
    )
  ) : (
    <Navigate href="/login" />
  );
};

export default Authenticated;

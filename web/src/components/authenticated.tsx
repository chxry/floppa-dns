import { useContext, ReactNode } from "react";
import { Navigate } from "react-router";

import { AuthContext } from "../auth";

const Authenticated = (props: { children: ReactNode }) => {
  const { user, token } = useContext(AuthContext);

  return token ? (
    user ? (
      props.children
    ) : (
      <p>loading</p>
    )
  ) : (
    <Navigate to="/login" />
  );
};

export default Authenticated;

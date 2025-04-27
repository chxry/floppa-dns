import { useContext } from "react";

import { AuthContext } from "..";
import { Authenticated } from "../components";

const Dashboard = () => {
  const { user } = useContext(AuthContext);

  return (
    <Authenticated>
      <h2>hi {user}</h2>
    </Authenticated>
  );
};

export default Dashboard;

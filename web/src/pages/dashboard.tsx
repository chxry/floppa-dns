import { Authenticated } from "../components";

import { user } from "..";

const Dashboard = () => {
  return (
    <Authenticated>
      <h2>hi {user()}</h2>
    </Authenticated>
  );
};

export default Dashboard;

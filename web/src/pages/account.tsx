import { useContext } from "react";
import { LogOut } from "lucide-react";

import { AuthContext, fetchAuth } from "../auth";
import { Button } from "../components";

const Account = () => {
  const { token, setToken, user, setUser } = useContext(AuthContext);

  const logout = async () => {
    await fetchAuth("/api/auth/logout", token!, {
      method: "POST",
    });
    setUser(null);
    setToken(null);
  };

  return (
    <>
      <section className="mb-2">
        <h2 className="text-2xl font-bold">Details:</h2>
        <p>
          <strong>Username: </strong> {user!.username}
        </p>
        <p>
          <strong>Created: </strong> {user!.created}
        </p>
      </section>
      <Button className="block" color="red" onClick={logout}>
        <LogOut className="inline mr-1" />
        Logout
      </Button>
    </>
  );
};

export default Account;

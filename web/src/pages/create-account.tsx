import { useState, useContext } from "react";
import { Navigate, useNavigate } from "react-router";

import { AuthContext } from "..";
import { Input, Button, Link } from "../components";

const CreateAccount = () => {
  const { token, setToken, user } = useContext(AuthContext);
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (token) {
    return <Navigate to="/dashboard" />;
  }

  const handler = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const res = await fetch("/api/auth/create-account", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          username: e.currentTarget.username.value,
          password: e.currentTarget.password.value,
        }),
      });

      if (res.status != 200) {
        setError("User already exists.");
        setLoading(false);
        return;
      }

      setToken(await res.text());
      navigate("/dashboard");
    } catch {
      setError("api error");
    }
    setLoading(false);
  };

  return (
    <form className="space-y-2" onSubmit={handler}>
      <h2 className="text-2xl font-bold">Create Account:</h2>
      <Input
        type="text"
        placeholder="Username"
        name="username"
        required
        maxlength={64}
        disabled={loading}
      />
      <Input
        type="password"
        placeholder="Password"
        name="password"
        required
        disabled={loading}
      />
      <Button disabled={loading}>
        {loading ? "Loading" : "Create Account"}
      </Button>
      {error && <p className="text-ctp-red">{error}</p>}
      <Link href="/login">Already have an account</Link>
    </form>
  );
};

export default CreateAccount;

import { useState, useContext } from "react";
import { Navigate, useNavigate } from "react-router";

import { AuthContext } from "../auth";
import { Input, Button, Link } from "../components";

const Login = () => {
  const { token, setToken } = useContext(AuthContext);
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
      const res = await fetch("/api/auth/login", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          username: e.currentTarget.username.value,
          password: e.currentTarget.password.value,
        }),
      });

      if (res.status !== 200) {
        setError("Invalid username or password.");
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
      <h2 className="text-2xl font-bold">Login:</h2>
      <Input
        className="block"
        type="text"
        placeholder="Username"
        name="username"
        required
        maxLength={64}
        disabled={loading}
      />
      <Input
        className="block"
        type="password"
        placeholder="Password"
        name="password"
        required
        disabled={loading}
      />
      <Button className="block" disabled={loading} long>
        {loading ? "Loading" : "Login"}
      </Button>
      {error && <p className="text-ctp-red">{error}</p>}
      <Link href="/create-account">Create an account</Link>
    </form>
  );
};

export default Login;

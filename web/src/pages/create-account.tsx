import { createSignal, JSX } from "solid-js";
import { useNavigate } from "@solidjs/router";

import { token, setToken } from "..";
import { Input, Button, Link } from "../components";

const CreateAccount = () => {
  const navigate = useNavigate();
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  if (token()) {
    navigate("/");
  }

  const handler: JSX.EventHandler<HTMLFormElement, SubmitEvent> = async (e) => {
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
    <form class="space-y-2" on:submit={handler}>
      <h2 class="text-2xl font-bold">Create Account:</h2>
      <Input
        type="text"
        placeholder="Username"
        name="username"
        required
        maxlength={64}
        disabled={loading()}
      />
      <Input
        type="password"
        placeholder="Password"
        name="password"
        required
        disabled={loading()}
      />
      <Button disabled={loading()}>
        {loading() ? "Loading" : "Create Account"}
      </Button>
      {error() && <p class="text-ctp-red">{error()}</p>}
      <Link href="/login">Already have an account</Link>
    </form>
  );
};

export default CreateAccount;

import { createSignal, createEffect, JSX } from "solid-js";
import { Router, Route, Navigate, useNavigate } from "@solidjs/router";
import { render } from "solid-js/web";

import "./index.css";
import { Link } from "./components";
import Home from "./pages/home";
import Login from "./pages/login";
import CreateAccount from "./pages/create-account";
import Dashboard from "./pages/dashboard";

export const [token, setToken] = createSignal(localStorage.getItem("token"));
export const [user, setUser] = createSignal<string | null>(null);

const App = () => {
  createEffect(() =>
    token()
      ? localStorage.setItem("token", token()!)
      : localStorage.removeItem("token"),
  );

  createEffect(async () => {
    if (token()) {
      try {
        const res = await fetch("/api/auth/user", {
          headers: { Authorization: `Bearer ${token()}` },
        });
        if (res.status != 200) {
          setToken(null);
          setUser(null);
          return;
        }

        setUser(await res.text());
      } catch {
        setToken(null);
        setUser(null);
      }
    }
  });

  return (
    <>
      <header class="bg-ctp-crust">
        <div class="max-w-4xl mx-auto flex items-center p-4">
          <h1 class="text-4xl font-bold flex-1">
            <a class="hover:text-ctp-lavender" href="/">
              floppa dns
            </a>
          </h1>
          {user() ? (
            <Link href="/dashboard">{user()}</Link>
          ) : (
            <Link href="/login">Login</Link>
          )}
        </div>
      </header>
      <main class="max-w-4xl mx-auto p-4">
        <Router>
          <Route path="/" component={() => <Home />} />
          <Route path="/login" component={() => <Login />} />
          <Route path="/create-account" component={() => <CreateAccount />} />
          <Route path="/dashboard" component={() => <Dashboard />} />
          <Route path="*" component={() => <Navigate href="/" />} />
        </Router>
      </main>
    </>
  );
};

render(() => <App />, document.getElementById("root")!);

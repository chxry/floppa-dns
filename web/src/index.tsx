import React, { useState, useEffect, createContext } from "react";
import { createRoot } from "react-dom/client";
import {
  BrowserRouter,
  Routes,
  Route,
  Navigate,
  Link as RouterLink,
} from "react-router";

import "./index.css";
import { Link } from "./components";
import Home from "./pages/home";
import Login from "./pages/login";
import CreateAccount from "./pages/create-account";
import Dashboard from "./pages/dashboard";

interface AuthContext {
  token: string | null;
  setToken: React.Dispatch<string | null>;
  user: string | null;
}

export const AuthContext = createContext<AuthContext>(undefined!);

const App = () => {
  const [token, setToken] = useState(localStorage.getItem("token"));
  const [user, setUser] = useState<string | null>(null);

  useEffect(
    () =>
      token
        ? localStorage.setItem("token", token)
        : localStorage.removeItem("token"),
    [token],
  );

  useEffect(() => {
    (async () => {
      if (token) {
        try {
          const res = await fetch("/api/auth/user", {
            headers: { Authorization: `Bearer ${token}` },
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
    })();
  }, [token]);

  return (
    <AuthContext.Provider value={{ token, setToken, user }}>
      <BrowserRouter>
        <header className="bg-ctp-crust">
          <div className="max-w-4xl mx-auto flex items-center p-4">
            <h1 className="text-4xl font-bold flex-1">
              <RouterLink className="hover:text-ctp-lavender" to="/">
                floppa dns
              </RouterLink>
            </h1>
            {user ? (
              <Link href="/dashboard">{user}</Link>
            ) : (
              <Link href="/login">Login</Link>
            )}
          </div>
        </header>
        <main className="max-w-4xl mx-auto p-4">
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/login" element={<Login />} />
            <Route path="/create-account" element={<CreateAccount />} />
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="*" element={<Navigate to="/" />} />
          </Routes>
        </main>
      </BrowserRouter>
    </AuthContext.Provider>
  );
};

createRoot(document.getElementById("root")!).render(<App />);

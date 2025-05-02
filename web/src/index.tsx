import React, { useState, useEffect, createContext } from "react";
import { createRoot } from "react-dom/client";
import {
  BrowserRouter,
  Routes,
  Route,
  Navigate,
  Link as RouterLink,
} from "react-router";
import { LogIn, User as UserIcon } from "lucide-react";

import "./index.css";
import { Link } from "./components";
import { User, AuthContext, fetchAuth } from "./auth";
import Home from "./pages/home";
import Login from "./pages/login";
import CreateAccount from "./pages/create-account";
import Dashboard from "./pages/dashboard";
import Account from "./pages/account";
import Domains from "./pages/domains";

type Info = {
  dns_zone: string;
};

export const InfoContext = createContext<Info>(undefined!);

const App = () => {
  const [token, setToken] = useState(localStorage.getItem("token"));
  const [user, setUser] = useState<User | null>(null);
  const [info, setInfo] = useState<Info | null>(null);

  useEffect(() => {
    if (token) {
      localStorage.setItem("token", token);
    } else {
      localStorage.removeItem("token");
    }

    (async () => {
      if (token) {
        try {
          const res = await fetchAuth("/api/auth/user", token!);
          if (res.status !== 200) {
            setToken(null);
            setUser(null);
            return;
          }

          setUser(await res.json());
        } catch {
          setToken(null);
          setUser(null);
        }
      }
    })();
  }, [token]);

  useEffect(() => {
    (async () => {
      const res = await fetch("/api/info");
      setInfo(await res.json());
    })();
  }, []);

  return (
    <AuthContext.Provider value={{ token, setToken, user, setUser }}>
      <BrowserRouter>
        <header className="bg-ctp-mantle">
          <div className="max-w-4xl mx-auto flex items-center p-4">
            <h1 className="text-4xl font-bold flex-1">
              <RouterLink className="hover:text-ctp-lavender" to="/">
                floppa dns
              </RouterLink>
            </h1>
            {user ? (
              <Link href="/dashboard">
                <UserIcon className="inline mr-1" />
                <span className="max-sm:hidden">{user.username}</span>
              </Link>
            ) : (
              <Link href="/login">
                <LogIn className="inline mr-1" />
                <span className="max-sm:hidden">Login</span>
              </Link>
            )}
          </div>
        </header>
        <main className="max-w-4xl mx-auto p-4">
          {info ? (
            <InfoContext.Provider value={info}>
              <Routes>
                <Route path="/" element={<Home />} />
                <Route path="/login" element={<Login />} />
                <Route path="/create-account" element={<CreateAccount />} />
                <Route path="/dashboard" element={<Dashboard />}>
                  <Route path="account" element={<Account />} />
                  <Route path="domains/:name?" element={<Domains />} />
                  <Route
                    index
                    path="*"
                    element={<Navigate to="/dashboard/account" />}
                  />
                </Route>
                <Route path="*" element={<Navigate to="/" />} />
              </Routes>
            </InfoContext.Provider>
          ) : (
            <p>loading</p>
          )}
        </main>
      </BrowserRouter>
    </AuthContext.Provider>
  );
};

createRoot(document.getElementById("root")!).render(<App />);

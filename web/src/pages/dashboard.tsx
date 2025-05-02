import { NavLink, Outlet } from "react-router";
import { UserRoundCog, Database, ShieldUser, LucideIcon } from "lucide-react";

import { Authenticated } from "../components";

const Dashboard = () => {
  return (
    <Authenticated>
      <div className="space-x-2 mb-4">
        <NavButton label="Account" icon={UserRoundCog} to="account" />
        <NavButton label="Domains" icon={Database} to="domains" />
        <NavButton label="Admin" icon={ShieldUser} to="admin" />
      </div>
      <Outlet />
    </Authenticated>
  );
};

const NavButton = (props: { label: string; icon: LucideIcon; to: string }) => {
  return (
    <NavLink
      className="p-2 rounded-md bg-ctp-mantle text-ctp-subtext0 hover:text-ctp-text [&.active]:text-ctp-lavender font-bold"
      to={`/dashboard/${props.to}`}
    >
      <props.icon className="inline mr-1 not-in-[&.active]:max-sm:mr-0" />
      <span className="not-in-[&.active]:max-sm:hidden">{props.label}</span>
    </NavLink>
  );
};

export default Dashboard;

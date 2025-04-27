import { Link as RouterLink } from "react-router";

const Link = (props: { href: string; children: React.ReactNode }) => {
  return (
    <RouterLink
      className="font-bold text-ctp-lavender underline"
      to={props.href}
    >
      {props.children}
    </RouterLink>
  );
};

export default Link;

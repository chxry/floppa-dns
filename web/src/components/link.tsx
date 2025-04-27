import { JSX } from "solid-js";

const Link = (props: { href: string; children: JSX.Element }) => {
  return (
    <a class="font-bold text-ctp-lavender underline" href={props.href}>
      {props.children}
    </a>
  );
};

export default Link;

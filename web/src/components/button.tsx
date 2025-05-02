const colors = {
  lavender:
    "bg-ctp-lavender text-ctp-base focus:ring-ctp-mantle enabled:hover:bg-ctp-lavender/80 disabled:bg-ctp-lavender/80 focus:ring-2 focus:ring-ctp-crust",
  red: "bg-ctp-mantle text-ctp-red focus:ring-ctp-red enabled:hover:bg-ctp-crust disabled:bg-ctp-crust focus:ring-2 focus:ring-ctp-red",
};

const Button = (props: {
  disabled?: boolean;
  onClick?: React.MouseEventHandler<HTMLButtonElement>;
  children: React.ReactNode;
  long?: boolean;
  color?: keyof typeof colors;
  className?: string;
}) => {
  return (
    <button
      className={`p-1 rounded-md outline-none focus:ring-2 font-bold cursor-pointer disabled:cursor-not-allowed ${colors[props.color || "lavender"]} ${props.long ? "w-full sm:w-72" : "px-2"} ${props.className || ""}`}
      disabled={props.disabled}
      onClick={props.onClick}
    >
      {props.children}
    </button>
  );
};

export default Button;

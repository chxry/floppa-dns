const Button = (props: { disabled?: boolean; children: React.ReactNode }) => {
  return (
    <button
      className="p-1 w-full sm:w-72 rounded-md block outline-none focus:ring-2 bg-ctp-lavender text-ctp-base focus:ring-ctp-mantle enabled:hover:bg-ctp-lavender/80 font-bold cursor-pointer disabled:cursor-disabled"
      disabled={props.disabled}
    >
      {props.children}
    </button>
  );
};

export default Button;

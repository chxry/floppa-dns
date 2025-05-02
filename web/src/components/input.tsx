const Input = (props: {
  type: string;
  placeholder?: string;
  name?: string;
  required?: boolean;
  maxLength?: number;
  defaultValue?: string;
  disabled?: boolean;
  className?: string;
}) => {
  return (
    <input
      className={`p-1 w-full sm:w-72 rounded-md outline-none focus:ring-2 bg-ctp-mantle text-ctp-subtext0 disabled:text-ctp-subtext1 focus:ring-ctp-lavender ${props.className || ""}`}
      type={props.type}
      placeholder={props.placeholder}
      name={props.name}
      required={props.required}
      maxLength={props.maxLength}
      defaultValue={props.defaultValue}
      disabled={props.disabled}
    />
  );
};

export default Input;

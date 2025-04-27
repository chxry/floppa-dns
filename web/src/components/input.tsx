const Input = (props: {
  type: string;
  placeholder?: string;
  name?: string;
  required?: boolean;
  maxlength?: number;
  disabled?: boolean;
}) => {
  return (
    <input
      class="p-1 w-full sm:w-72 rounded-md block outline-none focus:ring-2 bg-ctp-mantle text-ctp-subtext0 disabled:text-ctp-subtext1 focus:ring-ctp-lavender"
      type={props.type}
      placeholder={props.placeholder}
      name={props.name}
      required={props.required}
      maxlength={props.maxlength}
      disabled={props.disabled}
    />
  );
};

export default Input;

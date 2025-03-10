import "./../styles/Navbar.css";

export function Navbar({
	title,
  wallet,
  setWallet
}) {
	return <nav>
		<h1>{title}</h1>
    <button>[ connect ]</button>
	</nav>
}

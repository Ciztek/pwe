import axios from "axios";

const api = axios.create({
	baseURL: "http://127.0.0.1:8000",
});

export const getHello = async () => {
	const res = await api.get("/");
	return res.data;
};

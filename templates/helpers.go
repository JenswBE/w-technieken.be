package templates

type NavLink struct {
	Name     string
	URL      string
	Children []NavLink
}

type TemplateBaseCommon struct {
	NavLinks      []NavLink
	Email         string
	PhoneNumber   string
	VATNumber     string
	FacebookName  string
	FacebookLink  string
	InstagramName string
}

type TemplateBaseSpecific struct {
	CanonicalURL string
	CurrentLink  NavLink
	Title        string
}

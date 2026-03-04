function setup()
    default({
        recipe = Recipe.new,
        repository = "https://github.com/mbenoukaiss/ettac.git",
        keep_releases = 3,
        persistent_files = { ".env" },
        persistent_dirs = { "storage" },
    })

    host("with-password", {
        hostname = env("WITH_PASSWORD_HOSTNAME", "127.0.0.1"),
        port = 7022,
        user = "alice",
        password = env("PASSWORD"),
        path = "/",
    })

    host("with-public-key", {
        hostname = env("WITH_PUBLIC_KEY_HOSTNAME", "127.0.0.1"),
        port = 7122,
        user = "bob",
        private_key = env("PRIVATE_KEY"),
        path = "/",
    })
end

Recipe = {}

function Recipe:new()
    return setmetatable({}, self)
end

function Recipe:describe()
    use(System:new())
end

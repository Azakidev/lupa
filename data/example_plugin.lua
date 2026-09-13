--- Prefix to use in the query for explicit search
--- It is filtered out in the rust side before calling GET_RESULTS
--- It should be a char
PREFIX = '@'

--- The name of the plugin
--- It is used in the fallback provider filtering
--- Therefore it must never contain a comma
--- Filtering is case insensitive, but in the future it might appear in UI,
--- so keep it capitalized
NAME = "Example"

--- Whether the provider supports the sidebar or not
--- It defaults to false, but if it is true, the appropriate functions should be present
SUPPORT_SIDEBAR = false

--- The function for obtaining the results from a query
--- It should return a table of tables, each table containing the following information:
---
--- name: string
--- description: string | nil
--- icon: string | nil
---
--- @param query string
--- @return table
function GET_RESULTS(query)
    return {
        {
            name = query,
            description = "This is a description",
            icon = nil,
        },
    }
end

--- This function is run when the user confirms the entry
--- The window will close after it has run
--- @param entry table
function EXECUTE_ENTRY(entry)
    print("You executed " .. entry.name .. "!")
end

--- The function for obtaining the sidebar actions for a specific entry
--- It should return a table of tables, each table containing the following information:
---
--- name: string
--- icon: string | nil
---
--- @param entry_name string
--- @return table
function GET_SIDEBAR_ACTIONS(entry_name)
    return {
        {
            name = query,
            icon = nil,
        },
    }
end

--- This function is run when the user confirms a sidebar action to run
--- The window will close after it has run
--- @param entry table
function EXECUTE_SIDEBAR_ACTION(entry)
    print("You executed " .. entry.name .. "!")
end

defmodule Explorer.PolarsBackend do
  @moduledoc """
  The Explorer backend for Polars.
  """

  alias Explorer.PolarsBackend.Native

  @doc false
  def sql_execute(tables, sql_string) do
    # Extract the internal DataFrame data from each DataFrame
    tables_with_df =
      Enum.map(tables, fn {name, df} ->
        {name, df.data}
      end)

    with {:ok, polars_ldf} <- Native.sql_execute(tables_with_df, sql_string),
         {:ok, names} <- Native.lf_names(polars_ldf),
         {:ok, dtypes} <- Native.lf_dtypes(polars_ldf) do
      Explorer.Backend.DataFrame.new(polars_ldf, names, dtypes)
    else
      {:error, error} -> raise error
    end
  end
end

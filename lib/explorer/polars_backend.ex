defmodule Explorer.PolarsBackend do
  @moduledoc """
  The Explorer backend for Polars.
  """

  @behaviour Explorer.Backend

  alias Explorer.PolarsBackend.{Native, Shared}

  @impl true
  def sql_execute(tables, sql_string) do
    tables_with_df =
      Enum.map(tables, fn {name, df} ->
        {name, df.data}
      end)

    with {:ok, polars_ldf} <- Native.sql_execute(tables_with_df, sql_string),
         {:ok, polars_df} <- Native.lf_compute(polars_ldf) do
      Shared.create_dataframe!(polars_df)
    else
      {:error, error} -> raise error
    end
  end
end
